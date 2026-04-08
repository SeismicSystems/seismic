---
icon: browser
---

# Building the Frontend

In this section, you'll build a React frontend for the Clown Beatdown game. Players connect their wallet, punch the clown to reduce its stamina, and rob a shielded secret once it's knocked out — all from the browser. _Estimated time: ~45 minutes_

The frontend uses **seismic-react** to integrate shielded wallet functionality with a standard React + wagmi + RainbowKit stack. By the end of this section, you'll have a fully playable web app running against your local Seismic node.

### What You'll Learn

- Set up a React + Vite project with **seismic-react**, **wagmi**, and **RainbowKit**
- Configure providers for shielded wallet support
- Build custom hooks to interact with the ClownBeatdown contract
- Create game UI components with animations and responsive layout

### Overview of Chapters

- [**Ch 1: Project Setup and Providers**](chapter-1-project-setup-and-providers.md)

Install dependencies, configure Vite, and wire up the provider stack: WagmiProvider, RainbowKitProvider, and ShieldedWalletProvider from seismic-react.

- [**Ch 2: Contract Hooks**](chapter-2-contract-hooks.md)

Build the hooks that connect your UI to the ClownBeatdown contract — `useContract`, `useContractClient`, and `useGameActions`.

- [**Ch 3: Game UI Components**](chapter-3-game-ui-components.md)

Create the game interface: the clown sprite with punch animations, action buttons (hit, rob, reset), and the entry screen with wallet connection.
