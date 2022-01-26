#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg, MessagesResponse};
use crate::state::{ForumState, FORUM_STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:terra-trashtalk";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = ForumState {
        messages: Vec::new(),
        count: msg.count,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    FORUM_STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddMessage {message} => try_add_message(deps, message),
    }
}

pub fn try_add_message(deps: DepsMut, message: String) -> Result<Response, ContractError> {
    FORUM_STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.messages.push(message);
        state.count += 1;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_add_message"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetMessages {} => to_binary(&query_messages(deps)?)
    }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = FORUM_STATE.load(deps.storage)?;
    Ok(CountResponse { count: state.count })
}

fn query_messages(deps: Deps) -> StdResult<MessagesResponse> {
    let state = FORUM_STATE.load(deps.storage)?;
    Ok(MessagesResponse { messages: state.messages })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17, messages:vec![] };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn add_message() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { messages:vec![], count: 0 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::AddMessage {message: "Hello".to_string()};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::AddMessage {message: "Hello".to_string()};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(2, value.count);

        // should has new value in a vector
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMessages {}).unwrap();
        let value: MessagesResponse = from_binary(&res).unwrap();
        assert_eq!(vec!["Hello", "Hello"], value.messages);
    }

}
