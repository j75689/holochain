use holochain_p2p::HolochainP2pDna;
use holochain_state::host_fn_workspace::HostFnWorkspaceRead;
use holochain_state::prelude::SourceChainRead;
use holochain_types::prelude::*;
use holochain_zome_types::HeaderHashed;

use crate::core::ribosome::error::RibosomeResult;
use crate::core::ribosome::guest_callback::validation_package::ValidationPackageHostAccess;
use crate::core::ribosome::guest_callback::validation_package::ValidationPackageInvocation;
use crate::core::ribosome::guest_callback::validation_package::ValidationPackageResult;
use crate::core::ribosome::RibosomeT;
use crate::core::SourceChainResult;
use tracing::*;

pub async fn get_as_author_sub_chain(
    header_seq: u32,
    app_entry_type: AppEntryType,
    source_chain: &SourceChainRead,
) -> SourceChainResult<ValidationPackage> {
    // Collect and return the sub chain
    let elements = source_chain
        .query(
            ChainQueryFilter::default()
                .include_entries(true)
                .entry_type(EntryType::App(app_entry_type))
                .sequence_range(0..header_seq),
        )
        .await?;
    Ok(ValidationPackage::new(elements))
}

pub async fn get_as_author_full(
    header_seq: u32,
    source_chain: &SourceChainRead,
) -> SourceChainResult<ValidationPackage> {
    let elements = source_chain
        .query(
            ChainQueryFilter::default()
                .include_entries(true)
                .sequence_range(0..header_seq),
        )
        .await?;
    Ok(ValidationPackage::new(elements))
}

pub fn get_as_author_custom(
    header_hashed: &HeaderHashed,
    ribosome: &impl RibosomeT,
    network: &HolochainP2pDna,
    workspace_lock: HostFnWorkspaceRead,
) -> RibosomeResult<Option<ValidationPackageResult>> {
    let header = header_hashed.as_content();
    let access = ValidationPackageHostAccess::new(workspace_lock, network.clone());
    let app_entry_type = match header.entry_type() {
        Some(EntryType::App(a)) => a.clone(),
        _ => return Ok(None),
    };

    let zome = match ribosome
        .dna_def()
        .zomes
        .get(app_entry_type.zome_id().index())
    {
        Some(zome_tuple) => zome_tuple.clone().into(),
        None => {
            warn!(
                msg = "Tried to get custom validation package for header with invalid zome_id",
                ?header
            );
            return Ok(None);
        }
    };

    let invocation = ValidationPackageInvocation::new(zome, app_entry_type);

    Ok(Some(ribosome.run_validation_package(access, invocation)?))
}
