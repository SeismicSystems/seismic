---
icon: gamepad
---

# Ch 3: Game UI Components

In this chapter, you'll build the game interface — the clown sprite with punch animations, action buttons, and the entry screen. _Estimated time: ~15 minutes_

### ShowClown: Animated clown sprite

The clown sprite changes appearance based on how many times it's been hit. Create `src/components/game/ShowClown.tsx`:

```typescript
import { motion } from 'framer-motion'

interface ShowClownProps {
  isKO: boolean
  isShakingAnimation: boolean
  isHittingAnimation: boolean
  punchCount: number
}

export default function ShowClown({
  isKO,
  isShakingAnimation,
  isHittingAnimation,
  punchCount,
}: ShowClownProps) {
  // Select sprite based on game state
  const getClownImage = () => {
    if (isKO) return '/clownko.png'
    if (punchCount >= 2) return '/clown3.png'
    if (punchCount === 1) return '/clown2.png'
    return '/clown1.png'
  }

  return (
    <motion.img
      src={getClownImage()}
      alt="Clown"
      animate={{
        rotate: isShakingAnimation ? [0, -5, 5, -5, 5, 0] : 0,
        scale: isHittingAnimation ? [1, 0.9, 1.1, 1] : 1,
      }}
      transition={{
        duration: isShakingAnimation ? 0.5 : 0.3,
      }}
      style={{ width: '100%', maxWidth: '500px' }}
    />
  )
}
```

The sprite progression creates visual feedback as the clown takes damage:

- **0 hits** — `clown1.png` (full health)
- **1 hit** — `clown2.png` (damaged)
- **2–3 hits** — `clown3.png` (heavily damaged)
- **KO** — `clownko.png` (knocked out)

Framer Motion handles two animations: a **shake** (rotation) when the clown gets hit, and a **scale punch** effect for impact feedback.

### ButtonContainer: Action buttons

The button container renders the game's action buttons — hit, rob, and reset. Create `src/components/game/ButtonContainer.tsx`:

```typescript
import { useState } from 'react'

interface ButtonContainerProps {
  position: 'left' | 'right' | 'mobile'
  clownStamina: number | null
  isHitting: boolean
  isRobbing: boolean
  isResetting: boolean
  onHit: () => void
  onRob: () => void
  onReset: () => void
}

function ActionButton({
  onClick,
  defaultImg,
  activeImg,
  isActive,
  alt,
}: {
  onClick: () => void
  defaultImg: string
  activeImg: string
  isActive: boolean
  alt: string
}) {
  const [pressed, setPressed] = useState(false)

  return (
    <img
      src={pressed || isActive ? activeImg : defaultImg}
      alt={alt}
      onClick={onClick}
      onMouseDown={() => setPressed(true)}
      onMouseUp={() => setTimeout(() => setPressed(false), 200)}
      style={{
        cursor: 'pointer',
        width: '12rem',
        userSelect: 'none',
      }}
    />
  )
}

export default function ButtonContainer({
  position,
  clownStamina,
  isHitting,
  isRobbing,
  isResetting,
  onHit,
  onRob,
  onReset,
}: ButtonContainerProps) {
  const isKO = clownStamina === 0

  if (position === 'left' || (position === 'mobile' && !isKO)) {
    // Rob button (left side on desktop, shown on mobile when standing)
    return (
      <ActionButton
        onClick={onRob}
        defaultImg="/rob_btn.png"
        activeImg="/rob_active.png"
        isActive={isRobbing}
        alt="Rob"
      />
    )
  }

  // Right side: Hit when standing, Reset when KO
  if (isKO) {
    return (
      <ActionButton
        onClick={onReset}
        defaultImg="/reset_btn.png"
        activeImg="/reset_active.png"
        isActive={isResetting}
        alt="Reset"
      />
    )
  }

  return (
    <ActionButton
      onClick={onHit}
      defaultImg="/punch_btn.png"
      activeImg="/punch_active.png"
      isActive={isHitting}
      alt="Hit"
    />
  )
}
```

The button layout adapts based on the game state:

- **Clown standing** — show Hit button (right) and Rob button (left, will revert if called)
- **Clown KO** — show Reset button (right) and Rob button (left, now callable)

### ClownPuncher: Main game component

This component ties everything together. Create `src/components/game/ClownPuncher.tsx`:

```typescript
import { Box, CircularProgress, Typography } from '@mui/material'

import { useAuth } from '../chain/WalletConnectButton'
import ButtonContainer from './ButtonContainer'
import ShowClown from './ShowClown'
import { useGameActions } from '@/hooks/useGameActions'

export default function ClownPuncher() {
  const { isAuthenticated } = useAuth()
  const {
    loaded,
    clownStamina,
    isHitting,
    isResetting,
    isRobbing,
    robResult,
    punchCount,
    handleHit,
    handleReset,
    handleRob,
    setRobResult,
  } = useGameActions()

  if (!isAuthenticated) {
    return <EntryScreen />
  }

  if (!loaded || clownStamina === null) {
    return <CircularProgress />
  }

  const isKO = clownStamina === 0

  return (
    <Box sx={{ textAlign: 'center', position: 'relative' }}>
      {/* Secret revealed splash */}
      {robResult && (
        <Box
          onClick={() => setRobResult(null)}
          sx={{
            position: 'fixed',
            inset: 0,
            zIndex: 50,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            bgcolor: 'rgba(0,0,0,0.85)',
            cursor: 'pointer',
          }}
        >
          <Typography variant="h2">SECRET REVEALED!</Typography>
          <Typography variant="h4" sx={{ mt: 2 }}>
            {robResult}
          </Typography>
        </Box>
      )}

      {/* Game layout */}
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          gap: 4,
        }}
      >
        <ButtonContainer
          position="left"
          clownStamina={clownStamina}
          isHitting={isHitting}
          isRobbing={isRobbing}
          isResetting={isResetting}
          onHit={handleHit}
          onRob={handleRob}
          onReset={handleReset}
        />

        <ShowClown
          isKO={isKO}
          isShakingAnimation={isHitting}
          isHittingAnimation={isHitting}
          punchCount={punchCount}
        />

        <ButtonContainer
          position="right"
          clownStamina={clownStamina}
          isHitting={isHitting}
          isRobbing={isRobbing}
          isResetting={isResetting}
          onHit={handleHit}
          onRob={handleRob}
          onReset={handleReset}
        />
      </Box>
    </Box>
  )
}
```

### EntryScreen: Wallet connection

The entry screen prompts the user to connect their wallet before playing. Create `src/components/game/EntryScreen.tsx`:

```typescript
import { Box, Typography } from '@mui/material'

import { useAuth } from '../chain/WalletConnectButton'

export default function EntryScreen() {
  const { isLoading, openConnectModal } = useAuth()

  return (
    <Box
      onClick={openConnectModal}
      sx={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        height: '100vh',
        cursor: 'pointer',
      }}
    >
      <img src="/logo.png" alt="Clown Beatdown" style={{ maxWidth: '400px' }} />
      <Typography variant="h5" sx={{ mt: 4 }}>
        {isLoading ? '...Loading...' : 'CLICK TO CONNECT'}
      </Typography>
    </Box>
  )
}
```

Clicking anywhere on the entry screen opens the RainbowKit wallet connection modal. Once authenticated, the `ClownPuncher` component takes over.

### WalletConnectButton: Auth context

Create the auth context and wallet button used throughout the app. Create `src/components/chain/WalletConnectButton.tsx`:

```typescript
import { createContext, useContext, useState, useEffect } from 'react'
import { useAccount } from 'wagmi'
import { useConnectModal } from '@rainbow-me/rainbowkit'

interface AuthContextType {
  isAuthenticated: boolean
  isLoading: boolean
  openConnectModal: (() => void) | undefined
  accountName: string | null
}

const AuthContext = createContext<AuthContextType>({
  isAuthenticated: false,
  isLoading: true,
  openConnectModal: undefined,
  accountName: null,
})

export const useAuth = () => useContext(AuthContext)

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const { address, isConnected } = useAccount()
  const { openConnectModal } = useConnectModal()
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    setIsLoading(false)
  }, [address])

  const accountName = address
    ? `${address.slice(0, 6)}...${address.slice(-4)}`
    : null

  return (
    <AuthContext.Provider
      value={{
        isAuthenticated: isConnected,
        isLoading,
        openConnectModal,
        accountName,
      }}
    >
      {children}
    </AuthContext.Provider>
  )
}
```

### Running the frontend

Start the frontend dev server:

```bash
cd packages/web
bun dev
```

Make sure `sanvil` is running and the contract is deployed (see [Deploying](../writing-the-contract/deploying.md)). Open `http://localhost:5173` in your browser, connect your wallet, and start punching the clown!

### Game flow recap

1. **Connect wallet** — RainbowKit modal, ShieldedWalletProvider derives shielded keys
2. **Hit the clown** — `twrite.hit()` sends a shielded transaction, stamina decrements
3. **Clown KO** — stamina reaches 0, sprite changes to `clownko.png`
4. **Rob a secret** — `read.rob()` performs a signed read, secret is decrypted and displayed
5. **Reset** — `twrite.reset()` restores stamina and picks a new random secret for the next round

Congratulations! You've built a complete Seismic dApp — from smart contract to CLI to web frontend.
