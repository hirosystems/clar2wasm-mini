
use clarity::types::StacksEpochId;
use clarity::vm::ast::{self, ContractAST};
use clarity::vm::contexts::ContractContext;
use clarity::vm::contexts::GlobalContext;
use clarity::vm::costs::LimitedCostTracker;
use clarity::vm::database::MemoryBackingStore;
use clarity::vm::eval_all;
use clarity::vm::tests::test_only_mainnet_to_chain_id;
use clarity::vm::types::QualifiedContractIdentifier;
use clarity::vm::types::Value;
use clarity::vm::version::ClarityVersion;

use crate::CResult;

const VERSION: ClarityVersion = ClarityVersion::Clarity1;
const EPOCH: StacksEpochId = StacksEpochId::Epoch20;

pub(crate) fn parse_to_ast(source: &str) -> CResult<ContractAST> {
    let contract_id = QualifiedContractIdentifier::transient();

    Ok(ast::build_ast_with_rules(
        &contract_id,
        source,
        &mut (),
        VERSION,
        EPOCH,
        ast::ASTRules::PrecheckSize,
    )?)
}

pub(crate) fn eval_ast(ast: &ContractAST) -> CResult<Option<Value>> {
    let contract_id = QualifiedContractIdentifier::transient();
    let mut contract_context = ContractContext::new(contract_id.clone(), VERSION);
    let mut marf = MemoryBackingStore::new();
    let conn = marf.as_clarity_db();
    let chain_id = test_only_mainnet_to_chain_id(false);
    let mut global_context =
        GlobalContext::new(false, chain_id, conn, LimitedCostTracker::new_free(), EPOCH);
    Ok(global_context.execute(|g| eval_all(&ast.expressions, &mut contract_context, g, None))?)
}
