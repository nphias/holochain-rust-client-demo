
use holochain_client::{AdminWebsocket, AgentPubKey, AppWebsocket, AuthorizeSigningCredentialsPayload, ClientAgentSigner, ConductorApiError, InstalledAppId};
use holochain_conductor_api::CellInfo;
use holochain_types::prelude::{CellId, ExternIO};
use holochain_types::websocket::AllowedOrigins;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use indexmap::IndexMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct HelloOutput {
    pub message: String,
    pub author: AgentPubKey
}

#[derive(Debug,Clone)]
pub struct AppSessionData {
    pub admin_port: u16,
    pub app_id: InstalledAppId,
    pub cells: IndexMap<String,Vec<CellInfo>>,
}


impl AppSessionData {
    pub async fn init(app_id:String, admin_port:u16) -> Result<Self, ConductorApiError> {
        // Connect admin web socket
        let admin_ws = AdminWebsocket::connect((Ipv4Addr::LOCALHOST, admin_port))
            .await
            .map_err(|arg0: anyhow::Error| ConductorApiError::WebsocketError(holochain_websocket::WebsocketError::Other(arg0.to_string())))?;

        // Assume App is installed and enabled
        let appdata = admin_ws.enable_app(app_id.clone()).await?;
        let cell_data = appdata.app.cell_info;
        return Ok(Self{admin_port, app_id, cells: cell_data});
    }

    pub async fn zomecall(self, cell_id:CellId, zome_name:&str, fn_name:&str, payload:Option<ExternIO>) -> Result<ExternIO, ConductorApiError> {
        // ******** SIGNED ZOME CALL  ********
        let payload = match payload {
            Some(p) => p,
            None => ExternIO::encode(()).unwrap(),
        };
        // Connect admin web socket
        let admin_ws = AdminWebsocket::connect((Ipv4Addr::LOCALHOST, self.admin_port))
            .await
            .map_err(|arg0: anyhow::Error| ConductorApiError::WebsocketError(holochain_websocket::WebsocketError::Other(arg0.to_string())))?;

        let credentials = admin_ws
            .authorize_signing_credentials(AuthorizeSigningCredentialsPayload {
                cell_id: cell_id.clone(),
                functions: None,
            })
            .await
            .map_err(|arg0: anyhow::Error| ConductorApiError::WebsocketError(holochain_websocket::WebsocketError::Other(arg0.to_string())))?;
        let signer = ClientAgentSigner::default();
        signer.add_credentials(cell_id.clone(), credentials);

        // Connect app agent client
        let app_ws_port = admin_ws
          .attach_app_interface(0, AllowedOrigins::Any, None)
          .await?;

        let token_issued = admin_ws
          .issue_app_auth_token(self.app_id.clone().into())
          .await?;
        let app_ws = AppWebsocket::connect(
            (Ipv4Addr::LOCALHOST, app_ws_port),
            token_issued.token,
            signer.clone().into(),
        )
        .await
        .map_err(|arg0: anyhow::Error| ConductorApiError::WebsocketError(holochain_websocket::WebsocketError::Other(arg0.to_string())))?;

        // make zome call
        let response = app_ws
            .call_zome(
                cell_id.clone().into(),
                zome_name.into(),
                fn_name.into(),
                payload
            )
            .await?;
        Ok(response)
    }

    // Passing in None will return the first cell_id found, otherwise an error
    pub fn get_cell_id_by_role(&self, role: Option<&str>) -> Result<CellId,ConductorApiError> {
        if let Some(role) = role {
            if let Some(cell_data) = self.cells.get(role) {
                match cell_data[0].clone() {
                    CellInfo::Provisioned(c) => Ok(c.cell_id),
                    CellInfo::Cloned(c) => Ok(c.cell_id),
                    _ => Err(ConductorApiError::CellNotFound)
                }
            } else {
                Err(ConductorApiError::CellNotFound)
            }
        } else {
            if let Some(cell_data) = self.cells.values().next().clone() {
                match cell_data[0].clone() {
                    CellInfo::Provisioned(c) => Ok(c.cell_id),
                    CellInfo::Cloned(c) => Ok(c.cell_id),
                    _ => Err(ConductorApiError::CellNotFound)
                }
            } else {
                Err(ConductorApiError::CellNotFound)
            }
        }
        
    }
}

