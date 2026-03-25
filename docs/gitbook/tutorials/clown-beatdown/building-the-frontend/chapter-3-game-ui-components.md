---
icon: gamepad
---

# Ch 3: Game UI Components

In this chapter, you'll build the game interface — the clown sprite with punch animations, action buttons, and the entry screen. _Estimated time: ~15 minutes_

### ShowClown: Animated clown sprite

The clown sprite changes appearance based on how many times it's been hit. Create `src/components/game/ShowClown.tsx`:

```typescript
import { motion, useAnimation } from 'framer-motion'
import { useEffect, useMemo } from 'react'

import { Box } from '@mui/material'

type ClownProps = {
  isKO: boolean
  isShakingAnimation: boolean
  isHittingAnimation: boolean
  punchCount: number
}

const ShowClown: React.FC<ClownProps> = ({
  isKO,
  isShakingAnimation,
  isHittingAnimation,
  punchCount,
}) => {
  const controls = useAnimation()

  useEffect(() => {
    if (isShakingAnimation) {
      controls.start({
        rotate: [0, -5, 5, -5, 5, 0],
        transition: { duration: 0.5 },
      })
    } else if (isHittingAnimation) {
      controls.start({
        scale: [1, 0.9, 1.1, 1],
        transition: { duration: 0.3 },
      })
    }
  }, [isShakingAnimation, isHittingAnimation, controls])

  // Select the appropriate clown image based on punch count and KO state
  // Using useMemo to prevent recalculating on every render
  const clownImage = useMemo(() => {
    // If shaking, show the shaking clown image
    if (isShakingAnimation) {
      return '/clown_shaking.png'
    }

    if (isKO) {
      return '/clownko.png'
    }

    let imagePath
    switch (punchCount) {
      case 0:
        imagePath = '/clown1.png'
        break
      case 1:
        imagePath = '/clown2.png'
        break
      case 2:
      case 3:
        imagePath = '/clown3.png'
        break
      default:
        imagePath = '/clown1.png'
    }

    return imagePath
  }, [isKO, isShakingAnimation, punchCount])

  return (
    <Box
      sx={{
        width: { xs: '70%', sm: '70%', md: '80%', lg: '80%', xl: '100%' },
        height: { xs: '100%', sm: '70%', md: '80%', lg: '80%', xl: '100%' },
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
      }}
    >
      <div className="relative">
        <motion.div animate={controls} className="relative">
          <img
            src={clownImage}
            alt="Clown"
            style={{
              maxWidth: '100%',
              height: 'auto',
              objectFit: 'contain',
            }}
          />
        </motion.div>
      </div>
    </Box>
  )
}

export default ShowClown
```

The sprite progression creates visual feedback as the clown takes damage:

- **0 hits** — `clown1.png` (full health)
- **1 hit** — `clown2.png` (damaged)
- **2–3 hits** — `clown3.png` (heavily damaged)
- **KO** — `clownko.png` (knocked out)
- **Shaking** — `clown_shaking.png` (mid-animation)

Framer Motion's `useAnimation` hook controls two animations: a **shake** (rotation) when the clown gets hit, and a **scale punch** effect for impact feedback.

### ButtonContainer: Action buttons

The button container renders the game's action buttons — hit, rob, and reset. Create `src/components/game/ButtonContainer.tsx`:

```typescript
import { useState } from 'react'

import { Box, type SxProps, type Theme } from '@mui/material'

type ButtonContainerProps = {
  clownStamina: number | null
  isHitting: boolean
  isResetting: boolean
  isRobbing: boolean
  handleHit: () => void
  handleReset: () => void
  handleRob: () => void
  position?: 'left' | 'right' | 'mobile'
}

type ActionButtonProps = {
  onClick: () => void
  active: boolean
  src: string
  alt: string
  className: string
  sx?: SxProps<Theme>
}

const ActionButton = ({
  onClick,
  active,
  src,
  alt,
  className,
  sx,
}: ActionButtonProps) => (
  <Box
    onClick={onClick}
    component="div"
    sx={{
      cursor: active ? 'default' : 'pointer',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      ...sx,
    }}
  >
    <img
      src={src}
      alt={alt}
      className={className}
      style={{ width: '100%', height: '100%', objectFit: 'contain' }}
    />
  </Box>
)

export default function ButtonContainer({
  clownStamina,
  isHitting,
  isResetting,
  isRobbing,
  handleHit,
  handleReset,
  handleRob,
  position = 'mobile',
}: ButtonContainerProps) {
  const [showRobActive, setShowRobActive] = useState(false)
  const [showResetActive, setShowResetActive] = useState(false)

  const handleRobClick = () => {
    if (!isRobbing) {
      setShowRobActive(true)
      setTimeout(() => {
        setShowRobActive(false)
        handleRob()
      }, 200)
    }
  }

  const handleResetClick = () => {
    if (!isResetting) {
      setShowResetActive(true)
      setTimeout(() => {
        setShowResetActive(false)
        handleReset()
      }, 200)
    }
  }

  const isStanding = clownStamina !== null && clownStamina > 0

  const robBtn = {
    onClick: handleRobClick,
    active: isRobbing,
    src: showRobActive ? '/rob_active.png' : '/rob_btn.png',
    alt: 'Rob',
    className: 'look-btn',
  }

  const hitBtn = {
    onClick: handleHit,
    active: isHitting,
    src: isHitting ? '/punch_active.png' : '/punch_btn.png',
    alt: 'Punch',
    className: 'punch-btn',
  }

  const resetBtn = {
    onClick: handleResetClick,
    active: isResetting,
    src: showResetActive ? '/reset_active.png' : '/reset_btn.png',
    alt: 'Reset',
    className: 'reset-btn',
  }

  const rightBtn = isStanding ? hitBtn : resetBtn

  if (position === 'left') {
    return (
      <Box
        sx={{
          width: { lg: '100%' },
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'flex-end',
          pr: { lg: 4, xl: 6 },
        }}
      >
        <ActionButton
          {...robBtn}
          sx={{ width: '20rem', marginRight: 6, height: '20rem' }}
        />
      </Box>
    )
  }

  if (position === 'right') {
    return (
      <Box
        sx={{
          width: { lg: '100%' },
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'flex-start',
          pl: { lg: 4, xl: 6 },
        }}
      >
        <ActionButton
          {...rightBtn}
          sx={
            isStanding
              ? {
                  marginLeft: { xs: 0, lg: 8 },
                  height: { lg: '18rem' },
                }
              : { width: '20rem', marginLeft: '3rem', height: '18rem' }
          }
        />
      </Box>
    )
  }

  // Mobile layout — both buttons side by side
  const MOBILE_SIZE = {
    xs: '12rem',
    sm: '20rem',
    md: '20rem',
    lg: '30rem',
    xl: '30rem',
  }

  return (
    <Box
      sx={{
        width: '100%',
        height: '100%',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: { xs: 0, sm: 0, md: 0, lg: 70, xl: 70 },
        marginRight: { xs: 0, sm: 4, md: 4, lg: 6, xl: 0 },
        marginLeft: { xs: 0, sm: 4, md: 4, lg: 6, xl: 0 },
      }}
    >
      <ActionButton
        {...robBtn}
        sx={{ height: MOBILE_SIZE, width: MOBILE_SIZE }}
      />
      <ActionButton
        {...rightBtn}
        sx={
          isStanding
            ? {
                marginRight: { xs: 0, sm: 4, md: 0, lg: 0, xl: 0 },
                height: {
                  xs: '10rem',
                  sm: '18rem',
                  md: '20rem',
                  lg: '30rem',
                  xl: '30rem',
                },
                width: {
                  xs: '12rem',
                  sm: '14rem',
                  md: '28rem',
                  lg: '30rem',
                  xl: '30rem',
                },
              }
            : { height: MOBILE_SIZE, width: MOBILE_SIZE }
        }
      />
    </Box>
  )
}
```

The button layout adapts based on game state and screen size:

- **Desktop** — rob button on the left, hit/reset on the right
- **Mobile** — both buttons side by side below the clown
- **Clown standing** — show Hit button (right)
- **Clown KO** — show Reset button (right), Rob button now callable (left)

### ClownPuncher: Main game component

This component ties everything together. Create `src/components/game/ClownPuncher.tsx`:

```typescript
'use client'

import { useEffect, useRef, useState } from 'react'

import { useGameActions } from '@/hooks/useGameActions'
import {
  Backdrop,
  Box,
  CircularProgress,
  Container,
  Fade,
  Typography,
} from '@mui/material'

import { useAuth } from '../chain/WalletConnectButton'
import ButtonContainer from './ButtonContainer'
import EntryScreen from './EntryScreen'
import ShowClown from './ShowClown'

const ClownPuncher: React.FC = () => {
  const { isAuthenticated } = useAuth()
  const [showGame, setShowGame] = useState(false)
  const [showSecretSplash, setShowSecretSplash] = useState(false)
  const [showRobRefused, setShowRobRefused] = useState(false)
  const prevRoundIdRef = useRef<number | null>(null)
  const {
    loaded,
    currentRoundId,
    clownStamina,
    isHitting,
    isResetting,
    isRobbing,
    robResult,
    punchCount,
    fetchGameRounds,
    resetGameState,
    handleHit,
    handleReset,
    handleRob,
  } = useGameActions()

  useEffect(() => {
    // Only fetch data if authenticated and game is shown
    if (isAuthenticated && showGame) {
      fetchGameRounds()
    }
  }, [fetchGameRounds, isAuthenticated, showGame])

  useEffect(() => {
    // Only reset game state when first showing the game or when the round actually changes
    if (
      showGame &&
      (prevRoundIdRef.current === null ||
        (currentRoundId !== null && prevRoundIdRef.current !== currentRoundId))
    ) {
      console.log(
        'Round changed from',
        prevRoundIdRef.current,
        'to',
        currentRoundId,
        '- resetting game state'
      )
      resetGameState()
    }
    // Update the ref to the current round ID
    prevRoundIdRef.current = currentRoundId
  }, [currentRoundId, resetGameState, showGame])

  // Show splash screen when lookResult changes to a non-null value
  useEffect(() => {
    if (robResult !== null) {
      setShowSecretSplash(true)
    }
  }, [robResult])

  // If not showing the game yet, show entry screen
  if (!showGame) {
    return <EntryScreen onEnter={() => setShowGame(true)} />
  }

  const onRob = () => {
    if (clownStamina !== null && clownStamina > 0) {
      setShowRobRefused(true)
      return
    }
    handleRob()
  }

  const buttonProps = {
    clownStamina,
    isHitting,
    isResetting,
    isRobbing,
    handleHit,
    handleReset,
    handleRob: onRob,
  } as const

  return (
    <Container
      sx={{
        height: '100dvh',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'start',
        px: 4,
      }}
    >
      <Box
        sx={{
          mt: { xs: 3, sm: 3, md: 5, lg: 4, xl: 10 },
          height: {
            xs: '30dvh',
          },
          mb: 2,
          display: 'flex',
          justifyContent: 'center',
        }}
      >
        <img
          src="/cblogo.png"
          alt="Clown Beatdown Logo"
          className="clown-logo"
        />
      </Box>

      {/* Splash Screen — secret revealed or rob refused */}
      <Backdrop
        sx={{
          color: '#fff',
          zIndex: (theme) => theme.zIndex.drawer + 1,
          backgroundColor: 'rgba(0, 0, 0, 0.85)',
        }}
        open={(showSecretSplash && robResult !== null) || showRobRefused}
        onClick={() => {
          setShowSecretSplash(false)
          setShowRobRefused(false)
        }}
      >
        <Fade in={(showSecretSplash && robResult !== null) || showRobRefused}>
          <Box
            sx={{
              backgroundColor: 'background.paper',
              borderRadius: 4,
              p: 5,
              textAlign: 'center',
              maxWidth: '90%',
              boxShadow: 24,
            }}
          >
            {showRobRefused ? (
              <>
                <Typography
                  variant="h4"
                  fontWeight="bold"
                  color="white"
                  gutterBottom
                >
                  NOT SO FAST!
                </Typography>
                <Typography variant="h6" color="white" gutterBottom>
                  The clown isn't giving up that easily.
                </Typography>
                <Typography
                  variant="body1"
                  color="text.secondary"
                  sx={{ mt: 2 }}
                >
                  Knock him out first!
                </Typography>
              </>
            ) : (
              <>
                <Typography
                  variant="h4"
                  fontWeight="bold"
                  color="white"
                  gutterBottom
                >
                  SECRET REVEALED!
                </Typography>
                <Typography
                  variant="h1"
                  fontWeight="bold"
                  color="white"
                  gutterBottom
                >
                  {robResult}
                </Typography>
              </>
            )}
            <Typography variant="body1" color="text.secondary" sx={{ mt: 2 }}>
              (Click anywhere to close)
            </Typography>
          </Box>
        </Fade>
      </Backdrop>

      {loaded ? (
        <Box
          sx={{
            display: 'flex',
            flexDirection: { xs: 'column', lg: 'row' },
            justifyContent: { lg: 'space-between' },
            alignItems: 'center',
            width: '100%',
            position: 'relative',
            height: { lg: '500px', xl: '600px' },
            my: { xs: 0, md: 5, lg: 0, xl: 1 },
          }}
        >
          {/* Desktop: left buttons */}
          <Box sx={{ display: { xs: 'none', lg: 'flex' } }}>
            <ButtonContainer {...buttonProps} position="left" />
          </Box>

          {/* Clown — rendered once, responsive positioning */}
          <Box
            className="clown-container"
            sx={{
              display: 'flex',
              justifyContent: 'center',
              alignItems: 'center',
              position: { lg: 'absolute' },
              left: { lg: '50%' },
              transform: { lg: 'translateX(-50%)' },
              zIndex: 2,
              width: { lg: '50%', xl: '40%' },
              maxHeight: { xs: '35dvh', md: '30dvh', lg: 'none' },
            }}
          >
            <ShowClown
              isKO={clownStamina === 0}
              isShakingAnimation={false}
              isHittingAnimation={isHitting}
              punchCount={punchCount}
            />
          </Box>

          {/* Desktop: right buttons */}
          <Box sx={{ display: { xs: 'none', lg: 'flex' } }}>
            <ButtonContainer {...buttonProps} position="right" />
          </Box>

          {/* Mobile: all buttons below clown */}
          <Box
            sx={{
              display: { xs: 'flex', lg: 'none' },
              width: '100%',
              justifyContent: 'center',
              alignItems: 'center',
              marginBottom: { xs: 3, md: 5 },
            }}
          >
            <ButtonContainer {...buttonProps} position="mobile" />
          </Box>
        </Box>
      ) : (
        <CircularProgress size={32} />
      )}
    </Container>
  )
}

export default ClownPuncher
```

### EntryScreen: Wallet connection

The entry screen prompts the user to connect their wallet before playing. Create `src/components/game/EntryScreen.tsx`:

```typescript
import React, { useEffect, useState } from 'react'

import { Box, Container } from '@mui/material'

import { useAuth } from '../chain/WalletConnectButton'

type EntryScreenProps = {
  onEnter: () => void
}

const EntryScreen: React.FC<EntryScreenProps> = ({ onEnter }) => {
  const { isAuthenticated, isLoading, openConnectModal } = useAuth()
  const [isAnimating, setIsAnimating] = useState(false)

  // Automatically enter when user becomes authenticated
  useEffect(() => {
    if (isAuthenticated) {
      setIsAnimating(true)
      setTimeout(() => {
        setIsAnimating(false)
        onEnter()
      }, 500)
    }
  }, [isAuthenticated, onEnter])

  const handleLogoClick = () => {
    setIsAnimating(true)

    // If not authenticated, open wallet connect modal
    if (!isAuthenticated) {
      setTimeout(() => {
        setIsAnimating(false)
        openConnectModal()
      }, 300)
    }
  }

  return (
    <Container
      sx={{
        height: '100dvh',
        width: '100dvw',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        position: 'relative',
      }}
    >
      <Box
        sx={{
          display: 'flex',
          justifyContent: 'center',
          flexDirection: 'column',
          cursor: 'pointer',
          transform: isAnimating ? 'scale(0.95)' : 'scale(1)',
          transition: 'transform 0.2s ease-in-out',
        }}
        onClick={handleLogoClick}
      >
        <img
          src="/cblogo.png"
          alt="Clown Beatdown Logo"
          style={{ maxWidth: '100%', height: 'auto' }}
          className="clown-image"
        />
        <Box
          sx={{
            mt: 6,
            color: 'black',
            fontSize: '1.25rem',
            textAlign: 'center',
            opacity: 0.8,
            fontFamily: 'monospace',
            border: '1px solid black',
            borderRadius: '10px',
            padding: '10px',
            backgroundColor: 'var(--midColor)',
          }}
        >
          {isLoading ? '...Loading...' : 'CLICK TO CONNECT'}
        </Box>
      </Box>
    </Container>
  )
}

export default EntryScreen
```

Clicking the logo opens the RainbowKit wallet connection modal. Once authenticated, the `onEnter` callback fires and the `ClownPuncher` component takes over.

### WalletConnectButton: Auth context

Create the auth context and wallet button used throughout the app. Create `src/components/chain/WalletConnectButton.tsx`:

```typescript
import React, { createContext, useContext, useEffect, useState } from 'react'
import { useAccount } from 'wagmi'

import { ConnectButton } from '@rainbow-me/rainbowkit'
import { useConnectModal } from '@rainbow-me/rainbowkit'

// Create authentication context
type AuthContextType = {
  isAuthenticated: boolean
  isLoading: boolean
  openConnectModal: () => void
  accountName?: string
}

const AuthContext = createContext<AuthContextType>({
  isAuthenticated: false,
  isLoading: true,
  openConnectModal: () => {},
})

export const useAuth = () => useContext(AuthContext)

// Wallet icon component using SVG for better quality
const WalletIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 24 24"
    fill="currentColor"
    className="w-5 h-5"
  >
    <path d="M2.273 5.625A4.483 4.483 0 0 1 5.25 4.5h13.5c1.141 0 2.183.425 2.977 1.125A3 3 0 0 0 18.75 3H5.25a3 3 0 0 0-2.977 2.625ZM2.273 8.625A4.483 4.483 0 0 1 5.25 7.5h13.5c1.141 0 2.183.425 2.977 1.125A3 3 0 0 0 18.75 6H5.25a3 3 0 0 0-2.977 2.625ZM5.25 9a3 3 0 0 0-3 3v6a3 3 0 0 0 3 3h13.5a3 3 0 0 0 3-3v-6a3 3 0 0 0-3-3H15a.75.75 0 0 0-.75.75 2.25 2.25 0 0 1-4.5 0A.75.75 0 0 0 9 9H5.25Z" />
  </svg>
)

const WalletButton: React.FC<
  React.PropsWithChildren<
    { onClick: () => void } & React.HTMLAttributes<HTMLButtonElement>
  >
> = ({ children, onClick, ...props }) => {
  return (
    <button onClick={onClick} className="" {...props}>
      {children}
    </button>
  )
}

export const AuthProvider: React.FC<React.PropsWithChildren> = ({
  children,
}) => {
  const { openConnectModal } = useConnectModal() || {
    openConnectModal: () => {},
  }
  const { address, isConnecting, isConnected, isDisconnected } = useAccount()
  const [authState, setAuthState] = useState<AuthContextType>({
    isAuthenticated: false,
    isLoading: true,
    openConnectModal: openConnectModal || (() => {}),
  })

  useEffect(() => {
    setAuthState({
      isAuthenticated: isConnected,
      isLoading: isConnecting,
      openConnectModal: openConnectModal || (() => {}),
      accountName: address
        ? `${address.slice(0, 6)}...${address.slice(-4)}`
        : undefined,
    })
  }, [isConnected, isConnecting, isDisconnected, address, openConnectModal])

  return (
    <AuthContext.Provider value={authState}>{children}</AuthContext.Provider>
  )
}

const WalletConnectButton = () => {
  return (
    <ConnectButton.Custom>
      {({
        account,
        openConnectModal,
        chain,
        openAccountModal,
        openChainModal,
        mounted,
        authenticationStatus,
      }) => {
        if (!mounted || authenticationStatus === 'loading') {
          return <></>
        }
        if (!account || authenticationStatus === 'unauthenticated') {
          return (
            <WalletButton onClick={openConnectModal}>
              <span className="md:inline hidden">CONNECT WALLET</span>
              <span className="md:hidden">
                <WalletIcon />
              </span>
            </WalletButton>
          )
        }
        if (chain?.unsupported) {
          return (
            <WalletButton onClick={openChainModal}>
              <span className="md:inline hidden">Unsupported chain</span>
              <span className="md:hidden">
                <WalletIcon />
              </span>
            </WalletButton>
          )
        }
        return (
          <WalletButton onClick={openAccountModal}>
            <span className="">
              <WalletIcon />
            </span>
          </WalletButton>
        )
      }}
    </ConnectButton.Custom>
  )
}

export default WalletConnectButton
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
