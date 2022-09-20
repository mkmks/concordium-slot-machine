//! Slot machine smart contract.
//!
//! This smart contract module is developed as part of the
//!
//! Covers:
//! - Reading owner, sender, and self_balance from the context and host.
//! - The `ensure` macro.
//! - The `payable` attribute.
//! - The `mutable` attribute.
//! - Invoking a transfer with the host.

// Pulling in everything from the smart contract standard library.
use concordium_std::*;

/// The state of the slot machine
#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
enum SlotMachineEnum {
    /// People can insert some CCD to play a game.
    Intact,
    /// Someone played a game. Now waiting for oracle.
    ActiveGame,
    /// The slot machine paid out rewards.
    PaidOut,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
struct SlotMachineState {
    state: SlotMachineEnum,
    user_random_value: u8,
    oracle_random_value: u8,
}

/// Setup a new Intact slot machine.
#[init(contract = "SlotMachine")]
fn slot_machine_init<S: HasStateApi>(
    _ctx: &impl HasInitContext,
    _state_builder: &mut StateBuilder<S>,
) -> InitResult<SlotMachineState> {
    // Always succeeds
    Ok(SlotMachineState {
        state: SlotMachineEnum::Intact,
        user_random_value: 0,
        oracle_random_value: 0,
    })
}

/// Insert some CCD into a slot machine, allowed by anyone.
#[receive(contract = "SlotMachine", name = "insert", mutable, payable)]
fn slot_machine_insert<S: HasStateApi>(
    _ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<SlotMachineState, StateApiType = S>,
    amount: Amount,
) -> ReceiveResult<()> {
    // People have to pay 1 CCD to pull the lever of the slot machine
    ensure!(amount == Amount { micro_ccd: 1000000 });

    // Now a game is active and we wait for input from oracle
    host.state_mut().state = SlotMachineEnum::ActiveGame;

    host.state_mut().user_random_value = 50;

    Ok(())
}

/// View the state and balance of the slotmachine.
#[receive(contract = "SlotMachine", name = "view")]
fn slot_machine_view<S: HasStateApi>(
    _ctx: &impl HasReceiveContext,
    host: &impl HasHost<SlotMachineState, StateApiType = S>,
) -> ReceiveResult<(SlotMachineState, Amount)> {
    let current_state = *host.state();
    let current_balance = host.self_balance();
    Ok((current_state, current_balance))
}
