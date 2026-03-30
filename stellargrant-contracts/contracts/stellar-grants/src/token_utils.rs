use crate::{ContractError, Events};
use soroban_sdk::{symbol_short, Address, Env, IntoVal, InvokeError, String, Val, Vec};

fn emit_transfer_failed(
    env: &Env,
    grant_id: u64,
    token: &Address,
    from: &Address,
    to: &Address,
    amount: i128,
    reason: &str,
) {
    Events::emit_transfer_failed(
        env,
        grant_id,
        token.clone(),
        from.clone(),
        to.clone(),
        amount,
        String::from_str(env, reason),
    );
}

pub fn safe_balance(env: &Env, token: &Address, account: &Address) -> Result<i128, ContractError> {
    let mut args = Vec::new(env);
    args.push_back(account.into_val(env));

    match env.try_invoke_contract::<i128, InvokeError>(token, &symbol_short!("balance"), args) {
        Ok(Ok(balance)) => Ok(balance),
        Ok(Err(_)) | Err(_) => Err(ContractError::TokenBalanceQueryFailed),
    }
}

pub fn safe_transfer(
    env: &Env,
    grant_id: u64,
    token: &Address,
    from: &Address,
    to: &Address,
    amount: i128,
) -> Result<(), ContractError> {
    if amount <= 0 {
        return Err(ContractError::InvalidInput);
    }

    let balance = safe_balance(env, token, from)?;
    if balance < amount {
        return Err(ContractError::TokenTransferNotPossible);
    }

    let mut args: Vec<Val> = Vec::new(env);
    args.push_back(from.into_val(env));
    args.push_back(to.into_val(env));
    args.push_back(amount.into_val(env));

    match env.try_invoke_contract::<(), InvokeError>(token, &symbol_short!("transfer"), args) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(_)) => {
            emit_transfer_failed(
                env,
                grant_id,
                token,
                from,
                to,
                amount,
                "transfer_decode_failed",
            );
            Err(ContractError::TokenTransferFailed)
        }
        Err(_) => {
            emit_transfer_failed(
                env,
                grant_id,
                token,
                from,
                to,
                amount,
                "transfer_invoke_failed",
            );
            Err(ContractError::TokenTransferFailed)
        }
    }
}
