#!/usr/bin/env bash

dfx identity use minter

canister=$(dfx canister call buysomeoneacoffee get_principal)

dfx canister call icrc1_ledger_canister icrc1_transfer "(record { to = record { owner = $canister;};  amount = $1;})"