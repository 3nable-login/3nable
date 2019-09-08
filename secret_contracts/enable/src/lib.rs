
// Rust’s standard library provides a lot of useful functionality, but assumes support for various
// features of its host system: threads, networking, heap allocation, and others. SGX environments
// do not have these features, so we tell Rust that we don’t want to use the standard library
#![no_std]
#![allow(unused_attributes)]

#[macro_use]
extern crate serde_derive;
extern crate serde;
// The eng_wasm crate allows to use the Enigma runtime, which provides:
//     - Read from state      read_state!(key)
//     - Write to state       write_state!(key => value)
//     - Print                eprint!(...)
extern crate eng_wasm;

// The eng_wasm_derive crate provides the following
//     - Functions exposed by the contract that may be called from the Enigma network
//     - Ability to call functions of ethereum contracts from ESC
extern crate eng_wasm_derive;

// The asymmetric features of enigma_crypto
extern crate enigma_crypto;

// Serialization stuff
extern crate rustc_hex;

// eng_wasm
use eng_wasm::*;
use eng_wasm_derive::pub_interface;
use eng_wasm::{String, Vec};
use enigma_crypto::asymmetric::KeyPair;

///////////////////////////////////////////////////////////////////////////////////////////////////

static USERS: &str = "users";
static LOGINS: &str = "logins";

// Structs
#[derive(Serialize, Deserialize)]
pub struct User {
    userId: String,
    privateKey: Vec<u8>,
    publicKey: Vec<u8>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Login {
    userId: String,
    code: U256,
    valid: bool,
}

// Public struct Contract which will consist of private and public-facing secret contract functions
pub struct Contract;

// Private functions accessible only by the secret contract
impl Contract {
    fn getUsers() -> Vec<User> {
        read_state!(USERS).unwrap_or_default()
    }

    fn getLogins() -> Vec<Login> {
        read_state!(LOGINS).unwrap_or_default()
    }
}

// Public trait defining public-facing secret contract functions
#[pub_interface]
pub trait ContractInterface{
    fn addUser(userId: String, privateKey: Vec<u8>, publicKey: Vec<u8>);
    fn addLogin(userId: String, code: U256);
    fn signMessage(code: U256, message: String) -> (Vec<u8>, Vec<u8>);
}

// Implementation of the public-facing secret contract functions defined in the ContractInterface
// trait implementation for the Contract struct above
impl ContractInterface for Contract {
    #[no_mangle]
    fn addUser(userId: String, privateKey: Vec<u8>, publicKey: Vec<u8>) {
        let mut users = Self::getUsers();
        users.push(User {
            userId,
            privateKey,
            publicKey,
        });
        write_state!(USERS => users);
    }

    #[no_mangle]
    fn addLogin(userId: String, code: U256) {
        let users = Self::getUsers();
        // check that a user with that userId exists
        match users.iter().position(|u| u.userId == userId) {
            Some(index) => {
                let mut logins = Self::getLogins();
                logins.push(Login {
                    userId,
                    code,
                    valid: true,
                });
                write_state!(LOGINS => logins);
            },
            None => panic!("Error: User does not exist")
        }
        
    }

    #[no_mangle]
    fn signMessage(code: U256, message: String) -> (Vec<u8>, Vec<u8>) {
        let mut logins = Self::getLogins();
        let users = Self::getUsers();
        // check that a login instance with that code exists
        match logins.iter().position(|l| l.code == code) {
            Some(index) => {
                if logins[index].valid {
                    logins[index].valid = false;
                    let userId = &logins[index].userId;
                    let userIndex = users.iter().position(|u| &u.userId == userId).unwrap();
                    let user = &users[userIndex];
                    let mut private_key_slice: [u8; 32] = [0; 32];
                    private_key_slice.copy_from_slice(&user.privateKey);
                    let keys = KeyPair::from_slice(&private_key_slice).unwrap();
                    let signedMessage = keys.sign(message.as_bytes()).unwrap();
                    write_state!(LOGINS => logins);
                    (signedMessage.to_vec(), user.publicKey.clone())
                } else {
                    panic!("Error: Code not valid")
                }
            },
            None => panic!("Error: Code does not exist")
        }
    }
}