use super::*;
use collections::HashMap;
use gpui::{AppContext, Global, ModelContext};

pub fn init(client: Arc<Client>, cx: &mut AppContext) {
    let registry = cx.new_model(|cx| {
        let mut registry = LanguageModelRegistry::default();
        register_language_model_providers(&mut registry, client, cx);
        registry
    });
    cx.set_global(GlobalLanguageModelRegistry(registry));
}

fn register_language_model_providers(
    registry: &mut LanguageModelRegistry,
    client: Arc<Client>,
    cx: &mut ModelContext<LanguageModelRegistry>,
) {
    registry.register_provider(CloudLanguageModelProvider::new(client.clone(), cx), cx);
    registry.register_provider(
        AnthropicLanguageModelProvider::new(client.http_client(), cx),
        cx,
    );
    registry.register_provider(
        OpenAiLanguageModelProvider::new(client.http_client(), cx),
        cx,
    );
    registry.register_provider(
        OllamaLanguageModelProvider::new(client.http_client(), cx),
        cx,
    );
}

struct GlobalLanguageModelRegistry(Model<LanguageModelRegistry>);

impl Global for GlobalLanguageModelRegistry {}

#[derive(Default)]
pub struct LanguageModelRegistry {
    providers: HashMap<LanguageModelProviderName, Arc<dyn LanguageModelProvider>>,
}

impl LanguageModelRegistry {
    pub fn global(cx: &AppContext) -> Model<Self> {
        cx.global::<GlobalLanguageModelRegistry>().0.clone()
    }

    pub fn register_provider<T: LanguageModelProvider + LanguageModelProviderState>(
        &mut self,
        provider: T,
        cx: &mut ModelContext<Self>,
    ) {
        let name = provider.name(cx);

        provider.subscribe(cx).detach();

        self.providers.insert(name, Arc::new(provider));
    }

    pub fn available_models(&self, cx: &AppContext) -> Vec<AvailableLanguageModel> {
        self.providers
            .values()
            .flat_map(|provider| {
                provider
                    .provided_models(cx)
                    .into_iter()
                    .map(|model| AvailableLanguageModel {
                        provider: provider.name(cx),
                        model,
                    })
            })
            .collect()
    }

    pub fn model(
        &mut self,
        requested: &AvailableLanguageModel,
        cx: &mut AppContext,
    ) -> Result<Arc<dyn LanguageModel>> {
        let provider = self.providers.get(&requested.provider).ok_or_else(|| {
            anyhow::anyhow!("No provider found for name: {:?}", requested.provider)
        })?;

        provider.model(requested.model.id.clone(), cx)
    }

    pub fn provider(
        &self,
        name: &LanguageModelProviderName,
    ) -> Option<Arc<dyn LanguageModelProvider>> {
        self.providers.get(name).cloned()
    }
}
