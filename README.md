# Zendvo | Soroban Smart Contracts

This repository houses the Soroban smart contracts for Zendvo, a decentralized time-locked gifting platform built on Stellar. These contracts manage the time-locking logic and recipient validation.

Zendvo is a digital gifting platform that enables users to send cash gifts that remain completely hidden until a predetermined unlock date and time. By using the Stellar blockchain, Zendvo transforms digital money transfers into memorable experiences filled with mystery and anticipation.

## Project Structure

```text
zendvocontract/
├── contracts/
│   └── time_lock/          # Core gift escrow contract
│       ├── src/
│       │   ├── lib.rs      # Contract entry point & module definitions
│       │   ├── types.rs    # Custom data structures (Gift struct)
│       │   ├── errors.rs   # Contract-specific error codes
│       │   ├── constants.rs# Business rules (Fees, Limits)
│       │   └── test.rs     # Integration and unit tests
│       └── Cargo.toml      # Contract-specific dependencies
├── Cargo.toml              # Workspace configuration
└── README.md               # You are here
```

## Contract Architecture: `time_lock`

The `time_lock` contract acts as a non-custodial escrow for USDC gifts. It leverages Soroban's capabilities to ensure:

- **Trustless Escrow:** Funds are securely held by the contract logic, ensuring they cannot be moved by anyone (including the sender) until the unlock conditions are met.
- **Time-Locked Release:** Utilizes the Stellar network's ledger time to prevent the `claim_gift` function from succeeding before the sender's specified `unlock_timestamp`.
- **Fee Logic:** Automatically calculates and deducts the protocol fee (200 BPS / 2%) during the escrow creation process.
- **Validation:** Enforces gift amount limits ($5 - $1,000) at the blockchain level.

## Key Modules

- **`types.rs`**: Defines the `Gift` storage object, tracking ownership, amount, and timing.
- **`errors.rs`**: Standardized error codes (e.g., `NotUnlocked`, `AlreadyClaimed`).
- **`constants.rs`**: Shared configuration for fees and limits.
- **`test.rs`**: A robust test suite for simulating gift creation and withdrawal scenarios.

## Benefits to the Stellar Ecosystem

Zendvo showcases the power of Stellar through:

1.  **Stablecoin Infrastructure:** Utilizing **USDC** for value preservation, ensuring that the gift amount remains stable from creation to unlock.
2.  **Soroban Smart Contracts:** Implementing decentralized time-locking logic that prevents early withdrawal, providing a middleman-free guarantee of the "hidden" nature of the gift.
3.  **Low-Cost Transactions:** Leveraging Stellar's speed and near-zero fees to ensure that more of the sender's money reaches the recipient.
4.  **Real-World Utility:** Connecting blockchain technology directly to Nigerian bank accounts via local payout partners, driving adoption of Web3 solutions for real-world financial needs.
5.  **Financial Inclusion:** Providing a good on/off-ramp experience that bridges global stablecoin liquidity with local financial systems.

## Who is Zendvo For?

- **Diaspora Senders:** Specifically targeting young adults (18-35) in the US, UK, and Canada looking for a more meaningful way to send money home to Nigeria.
- **Domestic Gifting (Future Phase):** Nigerians sending to Nigerians for birthdays, anniversaries, and holidays where surprise is key.
- **Memorable Occasions:** Perfect for Valentine's Day, graduations, and surprise celebrations where the timing of the gift is as important as the gift itself.

## Development Guide

### Prerequisites

- **Rust Toolchain**: `rustup` with `wasm32-unknown-unknown` target.
- **Stellar CLI**: Necessary for network interaction and local simulation.

### Building

Compile the contract to optimized WASM:

```bash
cargo build --target wasm32-unknown-unknown --release
```

### Testing

Verify the contract logic locally:

```bash
cargo test
```

## Integration

The contracts serve as the backend execution layer for the [Zendvo Web App](../zendvo). The App's `src/lib/stellar` module interacts with these contracts using the `stellar-sdk`.
