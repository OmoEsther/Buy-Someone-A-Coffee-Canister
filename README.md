# BuY SoMeOnE A CoFfeE IcP DapP

Welcome to Buy Someone a coffee Daapp project. A cool application that allows you to send cute messages and tips to any person you fancy on the ICP blockchain.

The DApp uses the [ICRC-1 token standard Legder](https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/) to power it's tipping process.

To learn more before you start working with buy someone a coffee dapp, see the following documentation available online:

- [Quick Start](https://internetcomputer.org/docs/quickstart/quickstart-intro)
- [SDK Developer Tools](https://internetcomputer.org/docs/developers-guide/sdk-guide)
- [Rust Canister Devlopment Guide](https://internetcomputer.org/docs/rust-guide/rust-intro)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/candid-guide/candid-intro)

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background --clean
```

Next we are going to create the following identities on our dfx instance, these identities make the creation of the ledger seamless. For more information about the identities check the [ICRC1 tutorial](https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/icrc1-ledger-setup)

```bash
# The minter identity
dfx identity new minter

# The archive controller
dfx identity new archive_controller
```


Then we proceed to deploy the ICRC1 Ledger, a script has been supplied for that. This sets up the ledger.

```bash
npm run deploy-ledger
```

After that you can then run the deploy script of the buy someone a coffee canister.

```bash
npm run gen-deploy
```

And lastly we run the faucet script which mints new tokens to our coffee canister

```bash
npm run faucet
```

Now you can test the coffee canister on candid ui
example link

```bash
http://127.0.0.1:4943/?canisterId={candid_ui_id}&id={coffee_canister}
```
