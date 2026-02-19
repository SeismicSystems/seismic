# Create project structure and monorepo workspace

1. **Create the project folder and navigate into it:**

```bash
mkdir walnut-app
cd walnut-app
```

2. **Create the `packages`  directory with subdirectories for `contracts` and `cli`**

```bash
mkdir -p packages/contracts packages/cli
```

The `contracts`  subdirectory will house the Seismic smart contract(s) and test(s) for the project, while the `cli` will house the interface to interact with the contracts.

3. **Initialize a bun project in the root directory:**

```bash
bun init -y && rm index.ts && rm tsconfig.json && touch .prettierrc && touch .gitmodules
```

We remove the default `index.ts` and `tsconfig.json` files created by `bun init -y`  to keep the root directory clean and focused on managing the monorepo structure rather than containing code. We also create a `.prettierrc` file for consistent code formatting and a `.gitmodules` file to manage contract submodules.

4. **Replace the default `package.json` with the following content for a monorepo setup:**

```json
{
  "workspaces": [
    "packages/**"
  ],
  "dependencies": {},
  "devDependencies": {
    "@trivago/prettier-plugin-sort-imports": "^5.2.1",
    "prettier": "^3.4.2"
  }
}
```

5. **Add the following to the `.prettierrc` file for consistent code formatting:**

```json
{
  "semi": false,
  "tabWidth": 2,
  "singleQuote": true,
  "printWidth": 80,
  "trailingComma": "es5",
  "plugins": ["@trivago/prettier-plugin-sort-imports"],
  "importOrder": [
    "<TYPES>^(?!@)([^.].*$)</TYPES>",
    "<TYPES>^@(.*)$</TYPES>",
    "<TYPES>^[./]</TYPES>",
    "^(?!@)([^.].*$)",
    "^@(.*)$",
    "^[./]"
  ],
  "importOrderParserPlugins": ["typescript", "jsx", "decorators-legacy"],
  "importOrderSeparation": true,
  "importOrderSortSpecifiers": true
}
```

6. **Replace the`.gitignore` file with:**

```gitignore
# Compiler files
cache/
out/

# Ignores development broadcast logs
!/broadcast
/broadcast/*/31337/
/broadcast/**/dry-run/

# Docs
docs/

# Dotenv file
.env

node_modules/
```

7. **Add the following to the `.gitmodules` file to track git submodules (in our case, only the Forge standard library, `forge-std`):**

```git
[submodule "packages/contracts/lib/forge-std"]
	path = packages/contracts/lib/forge-std
	url = https://github.com/foundry-rs/forge-std
```



