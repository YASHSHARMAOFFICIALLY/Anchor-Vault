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
        self.vault_state.vault_bump = bump.vault;
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
self.vault_state.vault_bump = bump.vault;
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

## Extra Study Notes From Discussion

This section explains the project in the same simple style as the questions we discussed.

## Is This A Smart Contract?

Yes. On Solana, smart contracts are usually called **programs**.

```text
Ethereum smart contract = Solidity contract
Solana smart contract   = Solana program
Anchor smart contract   = Solana program written with Anchor
```

In this project, the smart contract is here:

```text
programs/anchor-vault/src/lib.rs
```

It exposes these on-chain instructions:

```rust
pub fn initialize(ctx: Context<Initialize>) -> Result<()>
pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()>
pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()>
pub fn close(ctx: Context<Close>) -> Result<()>
```

Once deployed, users call these functions through Solana transactions.

## On-Chain vs Off-Chain

On-chain means it lives on Solana and is verified by the network.

In this project, these are on-chain:

```text
Anchor Rust program
vault PDA
vault_state PDA
vault SOL balance
transactions
```

Off-chain means it runs outside Solana.

Examples:

```text
Next.js frontend
React components
buttons and input fields
wallet connect UI
optional API server
database, if any
```

Simple flow:

```text
User clicks Deposit in frontend
  -> frontend builds Solana transaction
  -> wallet signs transaction
  -> transaction goes to Solana
  -> on-chain Anchor program runs
```

Short version:

```text
On-chain = rules and money movement
Off-chain = app/interface used to call those rules
```

## What Is The Vault?

The vault is a program-derived Solana account that holds SOL.

Current flow:

```text
User wallet -> vault PDA -> same user wallet
```

Right now, this project is mainly a learning/demo vault. The same user deposits and withdraws, so there is no special business rule yet.

The useful part is the pattern:

```text
money is held by a program-controlled account
program rules decide when money can leave
```

This pattern is used in:

```text
escrow
staking
savings vaults
game prize pools
DAO treasuries
time-locked accounts
```

A more useful future feature would be:

```text
user deposits SOL
user cannot withdraw until unlock_time
```

## User, Vault State, And Vault

There are three important addresses:

```text
1. user wallet address
2. vault_state PDA
3. vault PDA
```

### User Wallet

The user wallet is the normal wallet, like Phantom or Solflare.

In code:

```rust
#[account(mut)]
pub user: Signer<'info>,
```

Meaning:

```text
The user must sign the transaction.
The user's account can change because SOL may leave or rent may be paid.
```

### Vault State PDA

The vault state is a data account.

In code:

```rust
pub vault_state: Account<'info, ValutState>
```

It stores:

```rust
pub struct ValutState {
    pub vault_bump: u8,
    pub state_bump: u8,
}
```

So the vault state is like a notebook:

```text
stores information needed by the vault
```

### Vault PDA

The vault is the money holder.

In code:

```rust
pub vault: SystemAccount<'info>
```

It does not store custom `ValutState` data. It mainly holds SOL.

Short version:

```text
user        = wallet/signature
vault_state = data account/notebook
vault       = SOL holder/money box
```

## What Is A PDA?

PDA means **Program Derived Address**.

A normal wallet has:

```text
address + private key
```

A PDA has:

```text
address, but no private key
```

It is created from:

```text
seeds + program id + bump
```

In this project:

```rust
seeds = [b"state", user.key().as_ref()]
```

This creates the vault state PDA from:

```text
"state" + user wallet address
```

And:

```rust
seeds = [b"vault", vault_state.key().as_ref()]
```

This creates the vault PDA from:

```text
"vault" + vault_state address
```

Why PDA is useful:

```text
No human controls it with a private key.
Only the program can sign for it using correct seeds and bump.
```

That is why PDAs are used for vaults, escrow accounts, staking accounts, and treasuries.

## Macro vs Function

Anchor uses macros like:

```rust
#[account]
#[derive(Accounts)]
#[derive(InitSpace)]
#[program]
```

These are not normal functions.

```text
Function = runs at runtime
Macro    = runs at compile time and generates code
```

You do not call:

```rust
account()
```

You attach the macro to code:

```rust
#[account]
pub struct ValutState {
    pub vault_bump: u8,
    pub state_bump: u8,
}
```

Anchor macros generate common Solana code for account checking, serialization, instruction handling, and validation.

## `#[account]`

`#[account]` has two common uses.

### On A Data Struct

```rust
#[account]
pub struct ValutState {
    pub vault_bump: u8,
    pub state_bump: u8,
}
```

Meaning:

```text
This struct is on-chain account data.
```

Anchor can serialize it into bytes and deserialize it back into Rust data.

Anchor also adds an 8-byte discriminator at the start of account data.

That is why account space is:

```rust
space = 8 + ValutState::INIT_SPACE
```

### On An Account Field

```rust
#[account(mut)]
pub user: Signer<'info>
```

or:

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

Meaning:

```text
Apply these validation/creation rules to the account field directly below.
```

## `#[derive(InitSpace)]`

Used on data structs.

```rust
#[derive(InitSpace)]
#[account]
pub struct ValutState {
    pub vault_bump: u8,
    pub state_bump: u8,
}
```

It calculates how many bytes the struct needs.

Here:

```text
vault_bump = u8 = 1 byte
state_bump = u8 = 1 byte
```

So:

```text
ValutState::INIT_SPACE = 2 bytes
```

Total account space:

```text
8 discriminator bytes + 2 data bytes = 10 bytes
```

Without `InitSpace`, you would write:

```rust
space = 8 + 1 + 1
```

With `InitSpace`, you write:

```rust
space = 8 + ValutState::INIT_SPACE
```

## `#[derive(Accounts)]`

Used on instruction account structs.

```rust
#[derive(Accounts)]
pub struct Deposit<'info> {
    pub user: Signer<'info>,
    pub vault: SystemAccount<'info>,
    pub vault_state: Account<'info, ValutState>,
    pub system_program: Program<'info, System>,
}
```

Meaning:

```text
These are the accounts required to call this instruction.
Anchor should validate them before running the logic.
```

Short comparison:

| Code | Used On | Meaning |
| --- | --- | --- |
| `#[account]` above struct | Data struct | This is on-chain account data |
| `#[account(...)]` above field | Account field | Validate/create this account with these rules |
| `#[derive(InitSpace)]` | Data struct | Calculate account storage size |
| `#[derive(Accounts)]` | Instruction struct | Define required accounts for an instruction |

## `Account<'info, ValutState>`

This is an Anchor account type.

```rust
pub vault_state: Account<'info, ValutState>
```

Breakdown:

```text
Account   = Anchor wrapper for an on-chain account with structured data
'info     = Rust lifetime for account references during this instruction
ValutState = the data structure stored inside the account
```

So:

```text
Account<'info, ValutState>
```

means:

```text
An on-chain account whose data layout is ValutState.
```

`'info` is not a time limit like 10 seconds. It is Rust's way of proving the account references are valid while the instruction runs.

## `Signer<'info>`

```rust
pub user: Signer<'info>
```

Meaning:

```text
This account must sign the transaction.
```

Why?

For initialize:

```text
user pays to create vault_state
```

For deposit:

```text
SOL leaves the user's wallet
```

The program cannot spend from a user wallet unless the user signs.

`user` is just a field name. You can rename it to `userrr`, but then every reference must change.

`Signer` is a real Anchor type. You cannot rename or misspell it as `Signeer`.

## Initialize Account Creation

This block:

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

applies to the field directly below it:

```rust
pub vault_state: Account<'info, ValutState>
```

Meaning:

```text
Create a new PDA account called vault_state.
The user pays for it.
Its address is derived from "state" + user wallet.
It stores ValutState data.
Allocate enough space for ValutState.
```

Each part:

```text
init   = create the account now
payer  = user pays rent/fees for creation
seeds  = PDA address formula
bump   = Anchor finds/checks PDA bump
space  = bytes to allocate for account data
```

This is not for the user account. It is for `vault_state`.

The user is mentioned only because:

```rust
payer = user
```

## Initialize Logic

Current logic:

```rust
impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bump: &InitializeBumps) -> Result<()> {
        self.vault_state.vault_bump = bump.vault;
        self.vault_state.state_bump = bump.vault_state;

        Ok(())
    }
}
```

Breakdown:

```rust
impl<'info> Initialize<'info>
```

Adds methods to the `Initialize` accounts struct.

```rust
pub fn initialize
```

Defines a method named `initialize`.

```rust
&mut self
```

Gives mutable access to the accounts in `Initialize`.

Needed because this code writes into `vault_state`.

```rust
bump: &InitializeBumps
```

Function parameter. `bump` is the variable name.

`InitializeBumps` is generated by Anchor because the `Initialize` accounts use `bump`.

It is similar to:

```rust
pub struct InitializeBumps {
    pub vault_state: u8,
    pub vault: u8,
}
```

These lines:

```rust
self.vault_state.vault_bump = bump.vault;
self.vault_state.state_bump = bump.vault_state;
```

mean:

```text
Save the vault PDA bump into vault_state.vault_bump.
Save the vault_state PDA bump into vault_state.state_bump.
```

Why save bumps?

Later, withdraw and close need the vault bump to sign as the vault PDA.

```rust
Ok(())
```

means:

```text
finish successfully
```

## Deposit Instruction

Deposit moves SOL from the user wallet into the vault PDA.

Flow:

```text
user wallet -> vault PDA
```

Code shape:

```rust
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, ValutState>,

    pub system_program: Program<'info, System>,
}
```

### Deposit User

```rust
#[account(mut)]
pub user: Signer<'info>
```

Meaning:

```text
The user must sign.
The user's balance can change because SOL leaves the wallet.
```

### Deposit Vault

```rust
#[account(
    mut,
    seeds = [b"vault", vault_state.key().as_ref()],
    bump = vault_state.vault_bump,
)]
pub vault: SystemAccount<'info>
```

Meaning:

```text
This is the vault PDA.
It is mutable because its SOL balance increases.
Its address must match "vault" + vault_state.
Use the stored vault_bump to verify it.
```

`SystemAccount<'info>` means:

```text
normal Solana system account, mainly holding SOL, not custom Anchor data
```

### Deposit Vault State

```rust
#[account(
    seeds = [b"state", user.key().as_ref()],
    bump = vault_state.state_bump
)]
pub vault_state: Account<'info, ValutState>
```

Meaning:

```text
This is the user's vault_state PDA.
Its address must match "state" + user wallet.
It stores ValutState data.
```

### Deposit Logic

```rust
impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_account: Transfer<'_> = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(System::id(), cpi_account);
        transfer(cpi_ctx, amount)?;
        Ok(())
    }
}
```

Breakdown:

```rust
amount: u64
```

Amount of lamports to deposit.

```text
1 SOL = 1_000_000_000 lamports
```

```rust
let cpi_account: Transfer<'_> = Transfer {
    from: self.user.to_account_info(),
    to: self.vault.to_account_info(),
};
```

Prepares the transfer accounts:

```text
from = user wallet
to   = vault PDA
```

`to_account_info()` converts Anchor account wrappers into the lower-level account format needed by the System Program.

```rust
let cpi_ctx = CpiContext::new(System::id(), cpi_account);
```

Creates a CPI context.

CPI means:

```text
Cross Program Invocation
```

Here:

```text
anchor-vault program calls Solana System Program
```

```rust
transfer(cpi_ctx, amount)?;
```

Actually transfers lamports from user to vault.

The `?` means:

```text
if transfer fails, return the error immediately
if transfer succeeds, continue
```

```rust
Ok(())
```

The deposit completed successfully.

## Withdraw And Close

Deposit does not need PDA signer seeds because the user signs and SOL leaves the user wallet.

Withdraw and close do need PDA signer seeds because SOL leaves the vault PDA.

The vault has no private key, so the program signs with seeds:

```rust
let seeds = &[
    b"vault",
    self.vault_state.to_account_info().key.as_ref(),
    &[self.vault_state.vault_bump],
];
let signer_seeds = &[&seeds[..]];
```

Then Anchor uses:

```rust
CpiContext::new_with_signer(...)
```

Meaning:

```text
The program proves it controls the vault PDA.
Then it can transfer SOL from vault to user.
```

## Frontend Notes

Yes, this smart contract can have a Next.js frontend.

Normal Solana dapp setup:

```text
Rust Anchor program = on-chain smart contract/backend
Next.js frontend    = off-chain user interface
Phantom wallet      = signer
Solana RPC          = connection to blockchain
```

Frontend flow:

```text
connect wallet
derive vault_state PDA
derive vault PDA
call initialize/deposit/withdraw/close
show vault balance
```

The frontend derives the same PDAs:

```ts
const [vaultState] = PublicKey.findProgramAddressSync(
  [Buffer.from("state"), publicKey.toBuffer()],
  programId
);

const [vault] = PublicKey.findProgramAddressSync(
  [Buffer.from("vault"), vaultState.toBuffer()],
  programId
);
```

Then it calls instructions through the Anchor IDL:

```ts
await program.methods
  .deposit(new BN(1_000_000))
  .accounts({
    user: publicKey,
    vault,
    vaultState,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

## Rust Frontend Note

You can write frontend in Rust using frameworks like:

```text
Yew
Leptos
Dioxus
```

But for Solana dapps, Next.js/React is usually easier because wallet integrations are mostly JavaScript/TypeScript-first.

Rust frontend is possible, but harder:

```text
smaller ecosystem
fewer UI libraries
more setup
browser wallet integration is less direct
WebAssembly debugging can be harder
```

Use Rust frontend only if you specifically want to learn Rust web/WASM.

## Current Test Explanation

The current test checks the full vault lifecycle:

```text
initialize -> deposit -> withdraw -> close
```

It does this:

1. Starts a LiteSVM local Solana environment.
2. Loads the compiled `anchor_vault.so` program.
3. Creates a test user keypair.
4. Airdrops test SOL to the user.
5. Derives the vault_state PDA.
6. Derives the vault PDA.
7. Calls `initialize`.
8. Checks the vault starts with zero lamports.
9. Calls `deposit` with `1_000_000` lamports.
10. Checks the vault balance is `1_000_000`.
11. Calls `withdraw` with `400_000` lamports.
12. Checks the vault balance is `600_000`.
13. Calls `close`.
14. Checks the vault balance is `0`.

The test now prints useful output:

```text
User wallet address
Vault state PDA
Vault PDA
Deposit amount
Withdraw amount
Current vault balance
Available vault balance after deposit
Available vault balance after withdraw
Available vault balance after close
```

Run the test with printed output:

```bash
cargo test -- --nocapture
```

Build first if needed:

```bash
anchor build
```

If you run only:

```bash
cargo test
```

Rust hides `println!` output when tests pass.

## Are Tests Mandatory?

Strictly:

```text
not always mandatory
```

Professionally, for smart contracts:

```text
yes, you should write tests
```

Reason:

```text
smart contract bugs can lock or lose real funds
```

Start with useful tests:

```text
happy path: initialize -> deposit -> withdraw -> close
failure path: cannot withdraw too much
failure path: wrong user cannot withdraw
failure path: cannot deposit before initialize
failure path: cannot close someone else's vault
```

Tests are proof that the program does what the README says it does.

## Better Future Feature

The current vault is simple:

```text
same user deposits
same user withdraws
```

A better feature for a real submission/demo would be a time-lock vault.

Example state:

```rust
#[derive(InitSpace)]
#[account]
pub struct VaultState {
    pub owner: Pubkey,
    pub vault_bump: u8,
    pub state_bump: u8,
    pub unlock_time: i64,
}
```

Withdraw check:

```rust
require!(
    Clock::get()?.unix_timestamp >= self.vault_state.unlock_time,
    ErrorCode::VaultLocked
);
```

Then the app becomes:

```text
User deposits SOL into a savings vault.
User cannot withdraw before unlock_time.
Frontend shows locked/unlocked status.
```
