#[macro_use]
extern crate serde;
use candid::{Decode, Encode, Nat, Principal};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct CoffeeMessage {
    id: u64,
    name: String,
    message: String,
    receiver: String,
    timestamp: u64,
}

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for CoffeeMessage {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for CoffeeMessage {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static STORAGE: RefCell<StableBTreeMap<u64, CoffeeMessage, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct MessagePayload {
    name: String,
    message: String,
    amount: Nat,
    receiver: String,
}

#[derive(candid::CandidType, Serialize, Deserialize)]
struct TransferPayload {
    owner: Principal,
    amount: Nat,
}

#[derive(candid::CandidType, Serialize, Deserialize)]
struct BalancePayload {
    receiver: String,
}

#[ic_cdk::query]
fn get_message(id: u64) -> Result<CoffeeMessage, Error> {
    match _get_message(&id) {
        Some(message) => Ok(message),
        None => Err(Error::NotFound {
            msg: format!("a message with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
async fn get_account_balance() -> Result<Nat, String> {
    let balance = _get_balance(ic_cdk::api::id())
        .await
        .map_err(|e| format!("failed to call ledger: {:?}", e));
    balance
}

#[ic_cdk::update]
async fn get_coffee_balance(receiver: BalancePayload) -> Result<Nat, String> {
    let receiver_p = Principal::from_text(receiver.receiver).unwrap();
    let balance = _get_balance(receiver_p)
        .await
        .map_err(|e| format!("failed to call ledger: {:?}", e));
    balance
}

#[ic_cdk::update]
async fn send_message(message: MessagePayload) -> Result<Nat, String> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let receiver = Principal::from_text(message.receiver.clone()).unwrap();
    let payload = TransferPayload {
        owner: receiver,
        amount: message.amount,
    };
    let result = _transfer(payload)
        .await
        .map_err(|e| format!("failed to call ledger: {:?}", e))?
        .map_err(|e| format!("ledger transfer error {:?}", e));

    match result {
        Ok(value) => {
            let message = CoffeeMessage {
                id,
                name: message.name,
                message: message.message,
                receiver: message.receiver,
                timestamp: time(),
            };
            do_insert(&message);
            Ok(value)
        }
        Err(error) => Err(error),
    }
}

// helper method to perform insert.
fn do_insert(message: &CoffeeMessage) {
    STORAGE.with(|service| service.borrow_mut().insert(message.id, message.clone()));
}

#[ic_cdk::update]
fn delete_message(id: u64) -> Result<CoffeeMessage, Error> {
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(message) => Ok(message),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a message with id={}. message not found.",
                id
            ),
        }),
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

// a helper method to get a message by id. used in get_message/update_message
fn _get_message(id: &u64) -> Option<CoffeeMessage> {
    STORAGE.with(|service| service.borrow().get(id))
}

// a helper method to transfer the coffee amount to creator
async fn _transfer(transfer_args: TransferPayload) -> CallResult<Result<Nat, TransferError>> {
    let ledger_id = Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap();
    // The request object of the `icrc1_name` endpoint is empty.

    let args = TransferArg {
        from_subaccount: None,
        to: Account {
            owner: transfer_args.owner,
            subaccount: None,
        },
        fee: None,
        created_at_time: None,
        memo: None,
        amount: transfer_args.amount,
    };
    let (result,): (Result<Nat, TransferError>,) =
        ic_cdk::call(ledger_id, "icrc1_transfer", (args,)).await?;

    Ok(result)
}

// helper method to get balance
async fn _get_balance(account: Principal) -> CallResult<Nat> {
    let ledger_id = Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap();
    let args = Account {
        owner: account,
        subaccount: None,
    };
    // The request object of the `icrc1_name` endpoint is empty.
    let (result,): (Nat,) = ic_cdk::call(ledger_id, "icrc1_balance_of", (args,)).await?;
    Ok(result)
}

// a helper to help get canister principal
#[ic_cdk::query]
fn get_principal() -> Principal {
    ic_cdk::api::id()
}

// need this to generate candid
ic_cdk::export_candid!();
