use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::{
    desktop::{
        settings::{
            ColorScheme, Contrast, Namespace, ACCENT_COLOR_SCHEME_KEY, APPEARANCE_NAMESPACE,
            COLOR_SCHEME_KEY, CONTRAST_KEY,
        },
        Color,
    },
    zbus::object_server::SignalEmitter,
    zvariant::{OwnedValue, Value},
    PortalError,
};

#[async_trait]
pub trait SettingsImpl: Send + Sync {
    async fn read_all(
        &self,
        namespaces: Vec<String>,
    ) -> Result<HashMap<String, Namespace>, PortalError>;

    async fn read(&self, namespace: &str, key: &str) -> Result<OwnedValue, PortalError>;
}

pub(crate) struct SettingsInterface {
    imp: Arc<dyn SettingsImpl>,
    cnx: zbus::Connection,
}

impl SettingsInterface {
    pub fn new(imp: Arc<dyn SettingsImpl>, cnx: zbus::Connection) -> Self {
        Self { imp, cnx }
    }

    pub async fn changed(&self, namespace: &str, key: &str, value: Value<'_>) -> zbus::Result<()> {
        let object_server = self.cnx.object_server();
        let iface_ref = object_server
            .interface::<_, Self>(crate::proxy::DESKTOP_PATH)
            .await?;
        Self::setting_changed(iface_ref.signal_emitter(), namespace, key, value).await
    }

    pub async fn contrast_changed(&self, contrast: Contrast) -> zbus::Result<()> {
        self.changed(
            APPEARANCE_NAMESPACE,
            CONTRAST_KEY,
            OwnedValue::from(contrast).into(),
        )
        .await
    }

    pub async fn accent_color_changed(&self, color: Color) -> zbus::Result<()> {
        self.changed(
            APPEARANCE_NAMESPACE,
            ACCENT_COLOR_SCHEME_KEY,
            OwnedValue::try_from(color).unwrap().into(),
        )
        .await
    }

    pub async fn color_scheme_changed(&self, scheme: ColorScheme) -> zbus::Result<()> {
        self.changed(
            APPEARANCE_NAMESPACE,
            COLOR_SCHEME_KEY,
            OwnedValue::from(scheme).into(),
        )
        .await
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Settings")]
impl SettingsInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        2
    }

    #[zbus(out_args("value"))]
    async fn read_all(
        &self,
        namespaces: Vec<String>,
    ) -> Result<HashMap<String, Namespace>, PortalError> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Settings::ReadAll");

        let response = self.imp.read_all(namespaces).await;

        #[cfg(feature = "tracing")]
        tracing::debug!("Settings::ReadAll returned {:#?}", response);
        response
    }

    #[zbus(out_args("value"))]
    async fn read(&self, namespace: &str, key: &str) -> Result<OwnedValue, PortalError> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Settings::Read");

        let response = self.imp.read(namespace, key).await;

        #[cfg(feature = "tracing")]
        tracing::debug!("Settings::Read returned {:#?}", response);
        response
    }

    #[zbus(signal)]
    async fn setting_changed(
        signal_ctxt: &SignalEmitter<'_>,
        namespace: &str,
        key: &str,
        value: Value<'_>,
    ) -> zbus::Result<()>;
}
