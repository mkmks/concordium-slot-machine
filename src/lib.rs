//! Slot Machine smart contract.
//!
//! Allows anyone to insert CCD and have fun!
//!

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

/// The state of the slot machine
#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
struct SlotMachineState {
    user_randomness: u8,
    oracle_randomness: u8,
    state: SlotMachineEnum,
}

/// Setup a new Intact piggy bank.
#[init(contract = "SlotMachine")]
fn slot_machine_init<S: HasStateApi>(
    _ctx: &impl HasInitContext,
    _state_builder: &mut StateBuilder<S>,
) -> InitResult<SlotMachineState> {
    // Always succeeds
    Ok(SlotMachineState{user_randomness: 0, oracle_randomness: 0, state: SlotMachineEnum::Intact})
}

/// Play by inserting CCD and randomness, allowed by anyone.
#[receive(contract = "SlotMachine", name = "insert", payable, mutable)]
fn slot_insert<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<SlotMachineState, StateApiType = S>,
    amount: Amount,
) -> ReceiveResult<()> {
    let parameter: u8 = ctx.parameter_cursor().get()?; // todo: change type of randomness

    // People have to pay 1 CCD to pull the lever of the slot machine
    ensure!(amount == Amount { micro_ccd: 1000000 });

    // update randomness of user and set has_inserted to true
    (*host.state_mut()).user_randomness = parameter;
    (*host.state_mut()).state = SlotMachineEnum::ActiveGame;
    
    Ok(())
}

/// Add oracle randomness. Only allowed by owner of smart contract.
#[receive(contract = "SlotMachine", name = "oracle_insert", mutable)]
fn oracle_insert<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<SlotMachineState, StateApiType = S>,
) -> ReceiveResult<()> {
    // Get the contract owner, i.e. the account who initialized the contract.
    let owner = ctx.owner();
    // Get the sender, who triggered this function, either a smart contract or
    // an account.
    let sender = ctx.sender();

    // Ensure only the owner can smash the piggy bank.
    ensure!(sender.matches_account(&owner));


    let parameter: u8 = ctx.parameter_cursor().get()?; // todo: change type of randomness

    // update randomness of oracle
    (*host.state_mut()).user_randomness = parameter;
    
    Ok(())
}


/// View the state and balance of the piggy bank.
#[receive(contract = "SlotMachine", name = "view")]
fn piggy_view<S: HasStateApi>(
    _ctx: &impl HasReceiveContext,
    host: &impl HasHost<SlotMachineState, StateApiType = S>,
) -> ReceiveResult<(SlotMachineState, Amount)> {
    let current_state = *host.state();
    let current_balance = host.self_balance();
    Ok((current_state, current_balance))
}
