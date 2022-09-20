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
    WaitingForPlayer,
    /// Player started a game. Now waiting for oracle.
    ActiveGame,
    /// The slot machine paid out rewards.
    PayoutReady,
}

/// The state of the slot machine
#[derive(Debug, Serial, DeserialWithState, SchemaType)]
#[concordium(state_parameter = "S")]
struct SlotMachineState<S: HasStateApi> {
    players: StateMap<AccountAddress, u8, S>,
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
) -> InitResult<SlotMachineState<S>> {
    // Always succeeds
    Ok(SlotMachineState {
        players: _state_builder.new_map(),
        oracle_randomness: 0,
        state: SlotMachineEnum::WaitingForPlayer,
    })
}

/// Play by inserting CCD and randomness, allowed by anyone.
#[receive(contract = "SlotMachine", name = "insert", payable, mutable)]
fn slot_insert<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<SlotMachineState<S>, StateApiType = S>,
    amount: Amount,
) -> ReceiveResult<()> {
    // People have to pay 1 CCD to pull the lever of the slot machine
    ensure!(amount == Amount { micro_ccd: 1000000 });

    // Can only play if machine is waiting
    ensure!((*host.state_mut()).state == SlotMachineEnum::WaitingForPlayer);

    // update randomness and address of the player and set has_inserted to true
    if let Address::Account(player_address) = ctx.sender() {
        (*host.state_mut()).players.insert(player_address, 50); // todo: change to a random input of user
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
    host: &mut impl HasHost<SlotMachineState<S>, StateApiType = S>,
) -> ReceiveResult<()> {
    // Get the contract owner, i.e. the account who initialized the contract.
    let owner = ctx.owner();
    // Get the sender, who triggered this function, either a smart contract or
    // an account.
    let sender = ctx.sender();

    // Ensure only the owner can update the oracle.
    ensure!(sender.matches_account(&owner));

    let parameter: u8 = ctx.parameter_cursor().get()?;

    // if user is playing, transition to ready for payout
    if (*host.state_mut()).state == SlotMachineEnum::ActiveGame {
        (*host.state_mut()).state = SlotMachineEnum::PayoutReady;
    }

    // update randomness of oracle
    (*host.state_mut()).oracle_randomness = parameter;

    Ok(())
}

/// Check if the player won or not and payout
#[receive(contract = "SlotMachine", name = "receive_payout", mutable)]
fn receive_payout<S: HasStateApi>(
    _ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<SlotMachineState<S>, StateApiType = S>,
) -> ReceiveResult<()> {
    // Only possible if payout is ready
    ensure!((*host.state()).state == SlotMachineEnum::PayoutReady);

    if let Address::Account(player_address) = _ctx.sender() {
        let m = 10;
        let p = 2;
        if let Some(player_randomness) = (*host.state()).players.get(&player_address) {
            // check for payout
            if ((*host.state()).oracle_randomness % m + *player_randomness % m) % m <= p {
                // this game is done, wait for next player
                (*host.state_mut()).state = SlotMachineEnum::WaitingForPlayer;
                Ok(host.invoke_transfer(
                    &player_address,
                    Amount {
                        micro_ccd: 2_000_000,
                    },
                )?)
            } else {
                // this game is done, wait for next player
                (*host.state_mut()).state = SlotMachineEnum::WaitingForPlayer;
                Ok(())
            }
        } else {
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
    host: &impl HasHost<SlotMachineState<S>, StateApiType = S>,
) -> ReceiveResult<(u8, Amount)> {
    let current_balance = host.self_balance();
    Ok(((*host.state()).oracle_randomness, current_balance))
}
