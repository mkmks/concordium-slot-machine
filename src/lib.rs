//! Slot Machine smart contract.
//!
//! Allows anyone to insert CCD and have fun!
//!

// Pulling in everything from the smart contract standard library.
use concordium_std::*;

/// The state of the slot machine
#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy, SchemaType)]
enum SlotMachineEnum {
    /// People can insert some CCD to play a game.
    Intact,
    /// Someone played a game. Now waiting for oracle.
    ActiveGame,
    /// The slot machine paid out rewards.
    PaidOut,
}

/// The state of the slot machine
#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy, SchemaType)]
struct SlotMachineState {
    user_address: Option<AccountAddress>,
    user_randomness: u8,
    oracle_randomness: u8,
    state: SlotMachineEnum,
}

#[derive(Serialize, SchemaType)]
struct RandomValue {
    /// Random value
    random_value: u8,
}

/// Setup a new slot machine state.
#[init(contract = "SlotMachine")]
fn slot_machine_init<S: HasStateApi>(
    _ctx: &impl HasInitContext,
    _state_builder: &mut StateBuilder<S>,
) -> InitResult<SlotMachineState> {
    // Always succeeds
    Ok(SlotMachineState {
        user_address: None,
        user_randomness: 0,
        oracle_randomness: 0,
        state: SlotMachineEnum::Intact,
    })
}

/// Play by inserting CCD and randomness, allowed by anyone.
#[receive(contract = "SlotMachine", name = "insert", payable, mutable)]
fn slot_insert<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<SlotMachineState, StateApiType = S>,
    amount: Amount,
) -> ReceiveResult<()> {
    // People have to pay 1 CCD to pull the lever of the slot machine
    ensure!(amount == Amount { micro_ccd: 1000000 });

    // update randomness and address of the player and set has_inserted to true
    if let Address::Account(player) = ctx.sender() {
        (*host.state_mut()).user_address = Some(player);
        (*host.state_mut()).user_randomness = 50; // todo: change to a random input of user
        (*host.state_mut()).state = SlotMachineEnum::ActiveGame;
        Ok(())
    } else {
        Ok(())
    }
}

/// Add oracle randomness. Only allowed by owner of smart contract.
#[receive(
    contract = "SlotMachine",
    name = "oracle_insert",
    mutable,
    parameter = "RandomValue"
)]
fn oracle_insert<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<SlotMachineState, StateApiType = S>,
) -> ReceiveResult<()> {
    // Get the contract owner, i.e. the account who initialized the contract.
    let owner = ctx.owner();
    // Get the sender, who triggered this function, either a smart contract or
    // an account.
    let sender = ctx.sender();

    // Ensure only the owner can update the oracle.
    ensure!(sender.matches_account(&owner));

    let parameter: u8 = ctx.parameter_cursor().get()?; // todo: change type of randomness

    // update randomness of oracle
    (*host.state_mut()).user_randomness = parameter;

    Ok(())
}

/// Check if the player won or not
#[receive(contract = "SlotMachine", name = "receive_payout", mutable)]
fn receive_payout<S: HasStateApi>(
    _ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<SlotMachineState, StateApiType = S>,
) -> ReceiveResult<()> {
    let st = *host.state();
    let m = 10;
    let p = 2;
    if let Some(address) = st.user_address {
        if (st.oracle_randomness % m + st.user_randomness % m) % m <= p {
            (*host.state_mut()).state = SlotMachineEnum::PaidOut;
            Ok(host.invoke_transfer(
                &address,
                Amount {
                    micro_ccd: 2_000_000,
                },
            )?)
        } else {
            (*host.state_mut()).state = SlotMachineEnum::Intact;
            Ok(())
        }
    } else {
        Ok(())
    }
}

/// View the state and slot machine.
#[receive(contract = "SlotMachine", name = "view")]
fn slot_machine_view<S: HasStateApi>(
    _ctx: &impl HasReceiveContext,
    host: &impl HasHost<SlotMachineState, StateApiType = S>,
) -> ReceiveResult<(SlotMachineState, Amount)> {
    let current_state = *host.state();
    let current_balance = host.self_balance();
    Ok((current_state, current_balance))
}
