# Anchor Vault Revision Notes

These notes explain the current `anchor-vault` project in simple terms.

## Big Mental Model

An Anchor project has two main parts:

- `lib.rs` is the public menu of instructions users can call.
- `instructions/` contains the actual account rules and logic for each action.
- `state.rs` contains data structs stored on-chain.
- `Anchor.toml` is Anchor config: cluster, wallet, program id.
- `Cargo.toml` is Rust build/dependency config.
- `package.json` is JavaScript/TypeScript tooling config.

Simple rule:

```text
Verb/action?        instructions/
Noun/data account?  state.rs
Program entry?      lib.rs
Config?             Anchor.toml / Cargo.toml
```

Examples:

```text
initialize, deposit, withdraw -> instructions/
VaultState, Escrow, UserData  -> state.rs
```

## Folder Structure

```text
anchor-vault/
  Anchor.toml              Anchor config: cluster, wallet, program id
  Cargo.toml               Rust workspace config
  package.json             JS/TS tooling and dependencies

  programs/
    anchor-vault/
      Cargo.toml           Rust config for this one Solana program

      src/
        lib.rs             Main program entry point
        instructions.rs    Instruction index/export file
        instructions/
          initialize.rs    Initialize instruction accounts + logic
        state.rs           On-chain account data structs
        constants.rs       Shared constants
        error.rs           Custom errors

      tests/
        test_initialize.rs Rust tests

  target/                  Generated Rust/Anchor build output
  node_modules/            Downloaded JS dependencies
```

Do not study or manually edit `target/` or `node_modules/` first. They are generated/downloaded.

## `lib.rs`

Current shape:

```rust
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("...");

#[program]
pub mod anchor_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }
}
```

### `pub mod state;`

This tells Rust:

```text
Load the module/file called state.rs.
```

So:

```rust
pub mod state;
```

connects to:

```text
src/state.rs
```

### `pub use state::*;`

This re-exports public things from `state`.

Without it, you may need:

```rust
state::ValutState
```

With it, you can use:

```rust
ValutState
```

`*` means:

```text
everything public from this module
```

### `use anchor_lang::prelude::*;`

This imports common Anchor tools:

```text
Context
Result
Account
Signer
Program
System
Pubkey
```

Think of it as:

```text
Anchor starter toolkit import
```

### `declare_id!("...");`

This declares the Solana program address.

It should match the program id in `Anchor.toml`.

### `#[program]`

This is an Anchor macro.

It tells Anchor:

```text
The public functions inside this module are callable Solana instructions.
```

So:

```rust
pub fn initialize(...)
```

becomes an instruction users can call.

### `pub mod anchor_vault`

`anchor_vault` is a module, not a function and not a struct.

Breakdown:

```rust
pub mod anchor_vault
```

means:

```text
public module named anchor_vault
```

### `pub fn initialize(ctx: Context<Initialize>) -> Result<()>`

Breakdown:

- `pub` = public
- `fn` = function
- `initialize` = function/instruction name
- `ctx` = parameter name
- `Context<Initialize>` = Anchor context containing the accounts from `Initialize`
- `Result<()>` = success or error; `()` means no meaningful return value

## `instructions/initialize.rs`

Current shape:

```rust
use anchor_lang::prelude::*;
use crate::state::ValutState;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        seeds = [b"state", user.key().as_ref()],
        bump,
        space = 8 + ValutState::INIT_SPACE
    )]
    pub vault_state: Account<'info, ValutState>,

    #[account(
        seeds = [b"vault", vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bump: &InitializeBumps) -> Result<()> {
        self.vault_state.valut_bump = bump.vault;
        self.vault_state.state_bump = bump.vault_state;

        Ok(())
    }
}
```

### `use anchor_lang::prelude::*;`

Imports common Anchor types like:

```text
Account
Signer
SystemAccount
Program
System
Result
```

### `use crate::state::ValutState;`

Imports your on-chain data struct from `state.rs`.

`crate` means:

```text
this current Rust program/package
```

### `#[derive(Accounts)]`

Anchor macro that generates account validation code.

It checks things like:

- required signer
- mutable accounts
- PDA seeds
- account creation
- payer
- account space

### `pub struct Initialize<'info>`

Defines the account list for the `initialize` instruction.

`'info` is a Rust lifetime used by Anchor account references. For now, treat it as required syntax for account structs.

### `#[account(mut)] pub user: Signer<'info>`

The user wallet.

- `Signer` means the user must sign the transaction.
- `mut` means the account can change.

The user is mutable because they pay SOL to create the `vault_state` account.

### `vault_state`

```rust
#[account(
    init,
    payer = user,
    seeds = [b"state", user.key().as_ref()],
    bump,
    space = 8 + ValutState::INIT_SPACE
)]
pub vault_state: Account<'info, ValutState>
```

This creates a PDA account that stores `ValutState`.

Meanings:

- `init` = create this account
- `payer = user` = user pays rent/SOL for account creation
- `seeds = [...]` = derive PDA address from these bytes
- `bump` = Anchor finds the PDA bump
- `space = ...` = how many bytes to allocate
- `Account<'info, ValutState>` = this account stores `ValutState` data

`b"state"` means byte string.

`user.key().as_ref()` means:

```text
take user's public key and use it as bytes
```

### `vault`

```rust
#[account(
    seeds = [b"vault", vault_state.key().as_ref()],
    bump
)]
pub vault: SystemAccount<'info>
```

This validates a PDA system account.

It is usually used to hold SOL.

Difference:

```text
vault_state = data account
vault       = SOL holding account
```

### `system_program`

```rust
pub system_program: Program<'info, System>
```

Required because creating accounts uses Solana's System Program.

Since `vault_state` uses `init`, Anchor needs the System Program.

### `impl<'info> Initialize<'info>`

Adds helper methods to the `Initialize` account struct.

This lets you write logic like:

```rust
ctx.accounts.initialize(&ctx.bumps)
```

### `pub fn initialize(&mut self, bump: &InitializeBumps) -> Result<()>`

This method writes bump values into `vault_state`.

Breakdown:

- `&mut self` = this method can modify the accounts in `Initialize`
- `bump: &InitializeBumps` = Anchor-generated bump values
- `Result<()>` = success or error

### Writing the bumps

```rust
self.vault_state.valut_bump = bump.vault;
self.vault_state.state_bump = bump.vault_state;
```

This stores the PDA bump values inside your on-chain `vault_state` account.

### `Ok(())`

Means:

```text
success, with no return value
```

## Current Missing Piece

Your `lib.rs` calls:

```rust
initialize::handler(ctx)
```

So `initialize.rs` should usually have:

```rust
pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.initialize(&ctx.bumps)
}
```

Call chain:

```text
User calls initialize
  -> lib.rs initialize(ctx)
    -> initialize::handler(ctx)
      -> ctx.accounts.initialize(&ctx.bumps)
        -> writes bumps into vault_state
```

## How To Stop Depending On YouTube

Use this loop:

1. Read the file first.
2. Write the purpose of each line in plain English.
3. Predict what the compiler will complain about.
4. Run build/test.
5. Fix one error at a time.
6. Repeat.

For every new instruction, use this template:

```text
1. What action is this? initialize/deposit/withdraw?
2. Which accounts are needed?
3. Which accounts must sign?
4. Which accounts are mutable?
5. Which accounts are PDAs?
6. Which accounts are created?
7. What data changes?
8. What errors can happen?
```

Do not copy a full video silently. Copy one small block, then answer:

```text
What is this block doing?
Why does Solana need this account?
What changes after this instruction runs?
```

If you cannot answer those three, pause before writing more code.

## Quick Self-Test

1. Is `Initialize` a function or account struct?
2. Why does `user` need `Signer<'info>`?
3. Why is `user` marked `mut`?
4. What does `init` do?
5. Who pays for `vault_state`?
6. What data does `vault_state` store?
7. What is the `vault` account for?
8. Why does `system_program` exist?
9. What does `bump` help with?
10. What does `Ok(())` mean?

