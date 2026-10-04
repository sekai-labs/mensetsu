from pydantic import BaseModel, Field
from pydantic_settings import BaseSettings, SettingsConfigDict

from src.llm.roles import ModelRole


class RoleModelConfig(BaseModel):
    provider: str = Field(default="openai", description="openai | anthropic | openrouter | test")
    model_name: str = Field(default="gpt-4o-mini", description="Model name or identifier")
    temperature: float = Field(default=0.2, ge=0.0, le=2.0)
    max_tokens: int | None = Field(default=4000, gt=0)


class LLMSettings(BaseSettings):
    model_config = SettingsConfigDict(
        env_prefix="LLM_",
        env_nested_delimiter="__",
        extra="ignore",
    )

    default_provider: str = "openai"
    default_model: str = "gpt-4o-mini"
    openai_api_key: str | None = None
    openai_base_url: str | None = None
    anthropic_api_key: str | None = None
    openrouter_api_key: str | None = None
    openrouter_base_url: str = "https://openrouter.ai/api/v1"

                             
    fallback_chain: list[str] = Field(default_factory=lambda: ["openai", "anthropic", "openrouter", "mock"])

                           
    roles: dict[ModelRole, RoleModelConfig] = Field(
        default_factory=lambda: {
            ModelRole.EXTRACTION: RoleModelConfig(provider="openai", model_name="gpt-4o-mini", temperature=0.0),
            ModelRole.RESEARCH: RoleModelConfig(provider="openai", model_name="gpt-4o-mini", temperature=0.1),
            ModelRole.SYNTHESIS: RoleModelConfig(provider="openai", model_name="gpt-4o", temperature=0.2),
            ModelRole.MATCHING: RoleModelConfig(provider="openai", model_name="gpt-4o-mini", temperature=0.0),
            ModelRole.CURRICULUM: RoleModelConfig(provider="openai", model_name="gpt-4o", temperature=0.3),
            ModelRole.CONTENT: RoleModelConfig(provider="openai", model_name="gpt-4o", temperature=0.4),
            ModelRole.QUESTIONS: RoleModelConfig(provider="openai", model_name="gpt-4o", temperature=0.3),
            ModelRole.REFINEMENT: RoleModelConfig(provider="openai", model_name="gpt-4o-mini", temperature=0.1),
            ModelRole.VALIDATION: RoleModelConfig(provider="openai", model_name="gpt-4o-mini", temperature=0.0),
        }
    )

    def get_role_config(self, role: ModelRole) -> RoleModelConfig:
        if role in self.roles:
            return self.roles[role]
        return RoleModelConfig(provider=self.default_provider, model_name=self.default_model)


llm_settings = LLMSettings()
