---
description: Integrate Seismic with popular wallet connection libraries
icon: book-open
---

# Wallet Guides

`seismic-react` integrates with any wallet library that provides wagmi configuration. This section covers setup for the most popular options.

## Supported Libraries

| Library                         | Best For                         | Key Feature                                      |
| ------------------------------- | -------------------------------- | ------------------------------------------------ |
| **[RainbowKit](rainbowkit.md)** | dApps wanting polished wallet UI | Built-in modal, chain switching, account display |
| **[Privy](privy.md)**           | Apps needing email/social login  | Embedded wallets, onboarding flow                |
| **[AppKit](appkit.md)**         | WalletConnect ecosystem apps     | WalletConnect modal, broad wallet support        |

## Common Integration Pattern

Regardless of which wallet library you choose, the integration follows the same steps:

1. **Install** the wallet library alongside `seismic-react`
2. **Configure wagmi** with Seismic chain definitions
3. **Nest providers** in the correct order
4. **Use hooks** from `seismic-react` in your components

### Provider Nesting Order

All wallet integrations require the same provider hierarchy:

```
WagmiProvider
  └─ QueryClientProvider
       └─ [Wallet Provider] (RainbowKit / Privy / AppKit)
            └─ ShieldedWalletProvider
                 └─ Your App
```

{% hint style="warning" %}
`ShieldedWalletProvider` must be nested inside both `WagmiProvider` and your wallet provider. It reads the connected wallet from wagmi's context, so placing it outside will cause a runtime error.
{% endhint %}

### Chain Configuration

All wallet libraries use the same chain imports from `seismic-react/rainbowkit`:

```typescript
import { seismicTestnet } from "seismic-react/rainbowkit";
```

## Choosing a Library

- **Want the easiest setup with a beautiful wallet modal?** Use [RainbowKit](rainbowkit.md)
- **Need email/social login or embedded wallets?** Use [Privy](privy.md)
- **Want WalletConnect with maximum wallet compatibility?** Use [AppKit](appkit.md)

## Pages

| Page                            | Description                      |
| ------------------------------- | -------------------------------- |
| **[RainbowKit](rainbowkit.md)** | Setup with RainbowKit wallet UI  |
| **[Privy](privy.md)**           | Embedded wallets with Privy      |
| **[AppKit](appkit.md)**         | WalletConnect AppKit integration |

## See Also

- [Installation](../installation.md) -- Package setup and peer dependencies
- [ShieldedWalletProvider](../shielded-wallet-provider.md) -- Provider reference and configuration
- [Hooks Overview](../hooks/) -- All available hooks
