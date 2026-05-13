# Anchor Vault

Anchor Vault is a Solana program built with Anchor that allows a user to create a program-derived vault, deposit SOL into it, withdraw SOL from it, and close the vault by returning the remaining lamports to the user.

The project demonstrates PDA derivation, Anchor account validation, System Program CPI transfers, PDA signer seeds, and Rust-based local testing with LiteSVM.

## Program Overview

The program exposes four instructions:

| Instruction | Purpose |
| --- | --- |
| `initialize` | Creates the user's vault state account and derives the vault PDA. |
| `deposit` | Transfers lamports from the user wallet into the vault PDA. |
| `withdraw` | Transfers a requested amount of lamports from the vault PDA back to the user. |
| `close` | Transfers all remaining lamports from the vault PDA back to the user. |

Program ID:

```text
4wfrBFMu2p4yP8ffq11naUKCRKdiWqgyVNtdY2bqv3pc
```

## Architecture

```text
anchor-vault/
  Anchor.toml
  Cargo.toml
  package.json
  programs/
    anchor-vault/
      Cargo.toml
      src/
        lib.rs
        state.rs
        instructions.rs
        instructions/
          initialize.rs
          deposit.rs
          withdraw.rs
          close.rs
      tests/
        test_initialize.rs
```

Important files:

- `programs/anchor-vault/src/lib.rs` defines the public Anchor instruction entrypoints.
- `programs/anchor-vault/src/state.rs` stores the vault and state bump values.
- `programs/anchor-vault/src/instructions/initialize.rs` initializes the user-specific vault state PDA.
- `programs/anchor-vault/src/instructions/deposit.rs` transfers SOL into the vault.
- `programs/anchor-vault/src/instructions/withdraw.rs` transfers SOL out of the vault using PDA signer seeds.
- `programs/anchor-vault/src/instructions/close.rs` returns the vault's full balance to the user.
- `programs/anchor-vault/tests/test_initialize.rs` tests the complete initialize, deposit, withdraw, and close flow.

## PDA Design

The program uses two deterministic addresses:

| Account | Seeds | Description |
| --- | --- | --- |
| Vault state PDA | `["state", user]` | Stores the bump values required by the program. |
| Vault PDA | `["vault", vault_state]` | Holds the SOL deposited by the user. |

The vault PDA is a system account controlled by the program through signer seeds. Withdraw and close operations use the stored vault bump to sign CPIs from the vault PDA.

## Prerequisites

Install the following tools before running the project:

- Rust, using the pinned toolchain in `rust-toolchain.toml`
- Solana CLI
- Anchor CLI
- Yarn

The project is configured for localnet in `Anchor.toml`.

## Setup

From the project directory:

```bash
cd anchor-vault
yarn install
```

Build the Anchor program:

```bash
anchor build
```

Run the Rust test suite:

```bash
anchor test
```

The configured Anchor test script runs:

```bash
cargo test
```

## Local Development Commands

Check formatting:

```bash
yarn lint
```

Fix formatting:

```bash
yarn lint:fix
```

Run Rust tests directly:

```bash
cargo test
```

## Test Coverage

The included LiteSVM test validates the main user flow:

1. Initializes the vault state PDA and vault PDA.
2. Deposits `1_000_000` lamports into the vault.
3. Withdraws `400_000` lamports back to the user.
4. Closes the vault by transferring the remaining balance back to the user.
5. Confirms the vault balance is zero after close.

## Notes

- The vault currently supports native SOL deposits through System Program transfers.
- The vault is user-specific because the state PDA is derived from the user's public key.
- The state struct is named `ValutState` in the source code. It stores the same vault state data described in this README.
- This project is intended for local development and educational submission use.
