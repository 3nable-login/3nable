
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
use rustc_hex::ToHex;

///////////////////////////////////////////////////////////////////////////////////////////////////

static USERS: &str = "users";
static LOGINS: &str = "logins";

// Structs
#[derive(Serialize, Deserialize)]
pub struct User {
    user_id: String,
    private_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Login {
    user_id: String,
    code: U256,
    valid: bool,
}

// Public struct Contract which will consist of private and public-facing secret contract functions
pub struct Contract;

// Private functions accessible only by the secret contract
impl Contract {
    fn get_users() -> Vec<User> {
        read_state!(USERS).unwrap_or_default()
    }

    fn get_logins() -> Vec<Login> {
        read_state!(LOGINS).unwrap_or_default()
    }
}

// Public trait defining public-facing secret contract functions
#[pub_interface]
pub trait ContractInterface{
    fn add_user(user_id: String, private_key: Vec<u8>);
    fn add_login(user_id: String, code: U256);
    fn sign_message(code: U256, message: String) -> Vec<u8>;
}

// Implementation of the public-facing secret contract functions defined in the ContractInterface
// trait implementation for the Contract struct above
impl ContractInterface for Contract {
    #[no_mangle]
    fn add_user(user_id: String, private_key: Vec<u8>) {
        // create a user and push it into users
        let mut users = Self::get_users();
        users.push(User {
            user_id,
            private_key,
        });
        // update the state
        write_state!(USERS => users);
    }

    #[no_mangle]
    fn add_login(user_id: String, code: U256) {
        let users = Self::get_users();
        // check that a user with that user_id exists
        match users.iter().position(|u| u.user_id == user_id) {
            Some(_index) => {
                // create a login and push it into logins
                let mut logins = Self::get_logins();
                logins.push(Login {
                    user_id,
                    code,
                    valid: true,
                });
                //update the state
                write_state!(LOGINS => logins);
            },
            None => panic!("Error: User does not exist")
        }
        
    }

    #[no_mangle]
    fn sign_message(code: U256, message: String) -> Vec<u8> {
        let mut logins = Self::get_logins();
        let users = Self::get_users();
        // check that a login instance with that code exists
        match logins.iter().position(|l| l.code == code) {
            Some(_index) => {
                // check that code is still valid
                if logins[_index].valid {
                    // invalidate code
                    logins[_index].valid = false;
                    // retrieve userId from login
                    let user_id = &logins[_index].user_id;
                    // find user corresponding to that id
                    let user_index = users.iter().position(|u| &u.user_id == user_id).unwrap();
                    let user = &users[user_index];
                    // grap the private key corresponding to that user
                    let mut private_key_slice: [u8; 32] = [0; 32];
                    private_key_slice.copy_from_slice(&user.private_key);
                    let keys = KeyPair::from_slice(&private_key_slice).unwrap();
                    // sign the message using the private key
                    let signed_message = keys.sign(message.as_bytes()).unwrap();
                    // update the logins state
                    write_state!(LOGINS => logins);
                    // return the signed message and the public key
                    let mut result = keys.get_pubkey().to_vec();
                    let mut signature_suffix = signed_message.to_vec();
                    result.append(&mut signature_suffix);
                    let msg: String = result.to_hex();
                    eprint!("The pub key from the generated key material({})", msg);
                    result
                } else {
                    panic!("Error: Code not valid")
                }
            },
            None => panic!("Error: Code does not exist")
        }
    }
}