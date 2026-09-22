//! Comment-preserving writes to the context file.
//!
//! `ctx use` sets one key in a file a human may have hand-annotated, so the
//! rest of it — comments, key order, blank lines — must survive the write.
//! Each function loads the file through [`toml_edit::DocumentMut`] (an empty
//! document when the file does not exist), edits the keys it owns, re-parses
//! the result through [`Config`] to validate before anything reaches disk,
//! and only then writes. A missing file is created on a write and not on a
//! read — a read of an absent file is an empty config, which is what makes
//! `ctx list` work on a fresh machine.

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use seismic_tee_common::{Descriptors, NodeDescriptor};
use toml_edit::{DocumentMut, InlineTable, Item, Table, value};

use crate::Selection;
use crate::config::{Config, Network};

/// Set `current`, moving the old value to `previous`.
pub fn set_current(path: &Path, selection: &Selection) -> anyhow::Result<()> {
    edit(path, |doc| {
        if let Some(old) = doc.get("current").and_then(Item::as_str) {
            let old = old.to_string();
            doc["previous"] = value(old);
        }
        doc["current"] = value(selection.to_string());
        Ok(())
    })
}

/// Clear `current` (and leave `previous` alone, so `ctx use -` still works).
pub fn clear_current(path: &Path) -> anyhow::Result<()> {
    edit(path, |doc| {
        doc.remove("current");
        Ok(())
    })
}

/// Create or update `[networks.<name>]`: the pointer keys (dir, manifest,
/// source, network_id) become `network`'s; everything else in the table — its
/// `nodes`, and any comment a human left on it — stays.
pub fn set_network(path: &Path, name: &str, network: &Network) -> anyhow::Result<()> {
    edit(path, |doc| {
        let table = network_table(networks_table(doc)?, name)?;
        for key in POINTER_KEYS {
            table.remove(key);
        }
        for (key, pointer) in pointer_values(network) {
            table[key] = value(pointer);
        }
        Ok(())
    })
}

/// Replace `[networks.<name>.nodes]` with `nodes`, one inline table per
/// line; creates the network entry when absent. A node gone from the
/// provisioner's output is gone from the file — the import is the stack's
/// current truth, not a merge.
pub fn set_nodes(path: &Path, name: &str, nodes: &Descriptors) -> anyhow::Result<()> {
    edit(path, |doc| {
        let table = network_table(networks_table(doc)?, name)?;
        table.insert("nodes", Item::Table(nodes_table(nodes)));
        Ok(())
    })
}

/// The `[networks]` table, created implicit (no bare `[networks]` header) the
/// first time anything is written under it.
fn networks_table(doc: &mut DocumentMut) -> anyhow::Result<&mut Table> {
    doc.entry("networks")
        .or_insert_with(|| {
            let mut table = Table::new();
            table.set_implicit(true);
            Item::Table(table)
        })
        .as_table_mut()
        .context("`networks` in the context file is not a table")
}

/// `[networks.<name>]`, created the first time anything is written under it.
fn network_table<'a>(networks: &'a mut Table, name: &str) -> anyhow::Result<&'a mut Table> {
    networks
        .entry(name)
        .or_insert_with(|| Item::Table(Table::new()))
        .as_table_mut()
        .with_context(|| format!("`networks.{name}` in the context file is not a table"))
}

/// The keys that say where a network's artifact set is. `nodes` is not one:
/// [`set_nodes`] owns it.
const POINTER_KEYS: [&str; 4] = ["dir", "manifest", "source", "network_id"];

/// `network`'s pointer keys that are set, as strings for the file.
fn pointer_values(network: &Network) -> Vec<(&'static str, String)> {
    let path = |p: &PathBuf| p.to_string_lossy().into_owned();
    [
        ("dir", network.dir.as_ref().map(path)),
        ("manifest", network.manifest.as_ref().map(path)),
        ("source", network.source.clone()),
        ("network_id", network.network_id.clone()),
    ]
    .into_iter()
    .filter_map(|(key, v)| v.map(|v| (key, v)))
    .collect()
}

/// `nodes` as a `[networks.<name>.nodes]` table: one inline table per node.
fn nodes_table(nodes: &Descriptors) -> Table {
    let mut table = Table::new();
    for (name, descriptor) in nodes {
        table.insert(
            name,
            Item::Value(descriptor_inline_table(descriptor).into()),
        );
    }
    table
}

fn descriptor_inline_table(descriptor: &NodeDescriptor) -> InlineTable {
    let mut table = InlineTable::new();
    table.insert("public_ip", descriptor.public_ip.clone().into());
    table.insert("fqdn", descriptor.fqdn.clone().into());
    table
}

/// Load, edit, validate, write — the one shape every write in this module
/// follows.
fn edit(path: &Path, f: impl FnOnce(&mut DocumentMut) -> anyhow::Result<()>) -> anyhow::Result<()> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
    };
    let mut doc: DocumentMut = text
        .parse()
        .with_context(|| format!("{} is not valid TOML", path.display()))?;

    f(&mut doc)?;

    let written = doc.to_string();
    let config: Config = toml::from_str(&written)
        .with_context(|| format!("the write to {} would not parse back", path.display()))?;
    config
        .validate(path)
        .with_context(|| format!("the write to {} would be invalid", path.display()))?;

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::write(path, written).with_context(|| format!("writing {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(path: &Path) -> String {
        std::fs::read_to_string(path).unwrap()
    }

    fn descriptor(public_ip: &str, fqdn: &str) -> NodeDescriptor {
        NodeDescriptor {
            public_ip: public_ip.to_string(),
            fqdn: fqdn.to_string(),
        }
    }

    #[test]
    fn set_current_moves_the_old_value_to_previous() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            "current = \"devnet-1/alpha\"\n\n[networks.devnet-1]\ndir = \"/x\"\n",
        )
        .unwrap();

        set_current(
            &path,
            &Selection {
                network: "devnet-1".to_string(),
                node: Some("beta".to_string()),
            },
        )
        .unwrap();

        let text = read(&path);
        let config: Config = toml::from_str(&text).unwrap();
        assert_eq!(config.current.as_deref(), Some("devnet-1/beta"));
        assert_eq!(config.previous.as_deref(), Some("devnet-1/alpha"));
    }

    #[test]
    fn a_comment_above_a_network_table_survives_a_set_current() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            "# the founding network\n[networks.devnet-1]\ndir = \"/x\"\n",
        )
        .unwrap();

        set_current(
            &path,
            &Selection {
                network: "devnet-1".to_string(),
                node: None,
            },
        )
        .unwrap();

        assert!(read(&path).contains("# the founding network"));
    }

    #[test]
    fn set_network_keeps_the_tables_comment_and_its_nodes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            "# handed over by the founder\n[networks.partner-net]\nmanifest = \"/m.json\"\n\n\
             [networks.partner-net.nodes]\nmy-node = { public_ip = \"198.51.100.4\", fqdn = \
             \"n.example.com\" }\n",
        )
        .unwrap();

        set_network(
            &path,
            "partner-net",
            &Network {
                dir: Some("/networks/partner-net".into()),
                ..Default::default()
            },
        )
        .unwrap();

        let text = read(&path);
        assert!(text.contains("# handed over by the founder"), "{text}");
        let config: Config = toml::from_str(&text).unwrap();
        let network = &config.networks["partner-net"];
        assert_eq!(
            network.dir.as_deref(),
            Some(Path::new("/networks/partner-net"))
        );
        assert_eq!(network.manifest, None);
        assert_eq!(network.nodes.len(), 1);
        assert!(network.nodes.contains_key("my-node"));
    }

    #[test]
    fn a_write_that_would_produce_an_invalid_config_fails_without_touching_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let original = "current = \"devnet-1\"\n";
        std::fs::write(&path, original).unwrap();

        let err = set_network(
            &path,
            "devnet-1",
            &Network {
                dir: Some("/x".into()),
                manifest: Some("/x/network-manifest.json".into()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("would be invalid"), "{err}");
        assert_eq!(read(&path), original);
    }

    #[test]
    fn set_network_creates_or_replaces_one_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        set_network(
            &path,
            "devnet-1",
            &Network {
                dir: Some("/x".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let text = read(&path);
        assert!(
            !text.contains("[networks]\n"),
            "no bare [networks] header:\n{text}"
        );
        let config: Config = toml::from_str(&text).unwrap();
        assert_eq!(
            config.networks["devnet-1"].dir.as_deref(),
            Some(Path::new("/x"))
        );

        // A second write replaces the pointer keys rather than merging into
        // them.
        set_network(
            &path,
            "devnet-1",
            &Network {
                manifest: Some("/y/network-manifest.json".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let config: Config = toml::from_str(&read(&path)).unwrap();
        assert_eq!(config.networks["devnet-1"].dir, None);
        assert_eq!(
            config.networks["devnet-1"].manifest.as_deref(),
            Some(Path::new("/y/network-manifest.json"))
        );
    }

    #[test]
    fn set_nodes_on_a_fresh_file_creates_the_entry_and_writes_one_node_per_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let mut nodes = Descriptors::new();
        nodes.insert(
            "devnet-1-1".to_string(),
            descriptor("203.0.113.7", "devnet-1-1.seismicdev.net"),
        );
        nodes.insert(
            "devnet-1-2".to_string(),
            descriptor("203.0.113.8", "devnet-1-2.seismicdev.net"),
        );
        set_nodes(&path, "devnet-1", &nodes).unwrap();

        let text = read(&path);
        assert!(text.contains("[networks.devnet-1.nodes]"), "{text}");
        assert!(
            text.contains(
                "devnet-1-1 = { public_ip = \"203.0.113.7\", fqdn = \"devnet-1-1.seismicdev.net\" }"
            ),
            "{text}"
        );
        let config: Config = toml::from_str(&text).unwrap();
        assert_eq!(config.networks["devnet-1"].nodes.len(), 2);
        assert_eq!(
            config.networks["devnet-1"].nodes["devnet-1-1"].fqdn,
            "devnet-1-1.seismicdev.net"
        );
    }

    #[test]
    fn set_network_after_set_nodes_keeps_the_nodes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let mut nodes = Descriptors::new();
        nodes.insert(
            "alpha".to_string(),
            descriptor("203.0.113.7", "a.example.com"),
        );
        set_nodes(&path, "devnet-1", &nodes).unwrap();

        set_network(
            &path,
            "devnet-1",
            &Network {
                dir: Some("/x".into()),
                ..Default::default()
            },
        )
        .unwrap();

        let config: Config = toml::from_str(&read(&path)).unwrap();
        assert_eq!(
            config.networks["devnet-1"].dir.as_deref(),
            Some(Path::new("/x"))
        );
        assert_eq!(config.networks["devnet-1"].nodes.len(), 1);
    }

    #[test]
    fn a_second_set_nodes_drops_a_node_the_new_map_lacks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let mut nodes = Descriptors::new();
        nodes.insert(
            "alpha".to_string(),
            descriptor("203.0.113.7", "a.example.com"),
        );
        nodes.insert(
            "beta".to_string(),
            descriptor("203.0.113.8", "b.example.com"),
        );
        set_nodes(&path, "devnet-1", &nodes).unwrap();

        let mut fewer = Descriptors::new();
        fewer.insert(
            "alpha".to_string(),
            descriptor("203.0.113.7", "a.example.com"),
        );
        set_nodes(&path, "devnet-1", &fewer).unwrap();

        let config: Config = toml::from_str(&read(&path)).unwrap();
        assert_eq!(
            config.networks["devnet-1"].nodes.keys().collect::<Vec<_>>(),
            ["alpha"]
        );
    }

    #[test]
    fn clear_current_leaves_previous_and_networks_intact() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            "current = \"devnet-1\"\nprevious = \"devnet-0\"\n\n[networks.devnet-1]\ndir = \"/x\"\n",
        )
        .unwrap();

        clear_current(&path).unwrap();

        let config: Config = toml::from_str(&read(&path)).unwrap();
        assert_eq!(config.current, None);
        assert_eq!(config.previous.as_deref(), Some("devnet-0"));
        assert!(config.networks.contains_key("devnet-1"));
    }

    #[test]
    fn a_write_to_an_absent_file_creates_its_parent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/seismic/config.toml");

        set_current(
            &path,
            &Selection {
                network: "devnet-1".to_string(),
                node: None,
            },
        )
        .unwrap();

        assert!(path.is_file());
    }
}
