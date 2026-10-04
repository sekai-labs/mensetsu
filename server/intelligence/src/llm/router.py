import abc
import json
import logging
import os
import time
from typing import Any, Type, TypeVar

from pydantic import BaseModel

from src.llm.config import RoleModelConfig, llm_settings
from src.llm.roles import ModelRole
from src.llm.telemetry import telemetry

logger = logging.getLogger('mensetsu.llm.router')
T = TypeVar('T', bound=BaseModel)

class LLMProvider(abc.ABC):

    @abc.abstractmethod
    async def generate_structured(self, schema: Type[T], system_prompt: str, user_prompt: str, role: ModelRole, config: RoleModelConfig) -> T:
        pass

class OpenAIProvider(LLMProvider):

    def __init__(self, api_key: str | None=None, base_url: str | None=None):
        from openai import AsyncOpenAI
        key = api_key or os.environ.get('OPENAI_API_KEY') or 'mock-key'
        self.client = AsyncOpenAI(api_key=key, base_url=base_url)

    async def generate_structured(self, schema: Type[T], system_prompt: str, user_prompt: str, role: ModelRole, config: RoleModelConfig) -> T:
        t0 = time.perf_counter()
        response = await self.client.beta.chat.completions.parse(model=config.model_name, messages=[{'role': 'system', 'content': system_prompt}, {'role': 'user', 'content': user_prompt}], response_format=schema, temperature=config.temperature, max_tokens=config.max_tokens)
        latency = (time.perf_counter() - t0) * 1000.0
        parsed = response.choices[0].message.parsed
        prompt_tokens = response.usage.prompt_tokens if response.usage else 0
        comp_tokens = response.usage.completion_tokens if response.usage else 0
        telemetry.record_usage(role=role, provider='openai', model=config.model_name, prompt_tokens=prompt_tokens, completion_tokens=comp_tokens, latency_ms=latency)
        if parsed is None:
            raise ValueError('OpenAI failed to parse response into requested schema')
        return parsed

class AnthropicProvider(LLMProvider):

    def __init__(self, api_key: str | None=None):
        from anthropic import AsyncAnthropic
        key = api_key or os.environ.get('ANTHROPIC_API_KEY') or 'mock-key'
        self.client = AsyncAnthropic(api_key=key)

    async def generate_structured(self, schema: Type[T], system_prompt: str, user_prompt: str, role: ModelRole, config: RoleModelConfig) -> T:
        t0 = time.perf_counter()
        schema_json = json.dumps(schema.model_json_schema())
        augmented_prompt = f'{user_prompt}\n\nYou MUST reply with ONLY a valid JSON object strictly complying with this JSON Schema:\n{schema_json}\nOutput JSON only. Do not wrap in markdown or backticks.'
        response = await self.client.messages.create(model=config.model_name if 'claude' in config.model_name else 'claude-3-5-sonnet-20241022', system=system_prompt, messages=[{'role': 'user', 'content': augmented_prompt}], temperature=config.temperature, max_tokens=config.max_tokens or 4000)
        latency = (time.perf_counter() - t0) * 1000.0
        text = response.content[0].text.strip()
        if text.startswith('```json'):
            text = text[7:]
        if text.endswith('```'):
            text = text[:-3]
        text = text.strip()
        parsed = schema.model_validate_json(text)
        prompt_tokens = response.usage.input_tokens
        comp_tokens = response.usage.output_tokens
        telemetry.record_usage(role=role, provider='anthropic', model=config.model_name, prompt_tokens=prompt_tokens, completion_tokens=comp_tokens, latency_ms=latency)
        return parsed

class OpenRouterProvider(LLMProvider):

    def __init__(self, api_key: str | None=None, base_url: str='https://openrouter.ai/api/v1'):
        from openai import AsyncOpenAI
        key = api_key or os.environ.get('OPENROUTER_API_KEY') or 'mock-key'
        self.client = AsyncOpenAI(api_key=key, base_url=base_url)

    async def generate_structured(self, schema: Type[T], system_prompt: str, user_prompt: str, role: ModelRole, config: RoleModelConfig) -> T:
        t0 = time.perf_counter()
        schema_json = json.dumps(schema.model_json_schema())
        augmented_prompt = f'{user_prompt}\n\nYou MUST reply with ONLY a valid JSON object strictly complying with this JSON Schema:\n{schema_json}\nOutput JSON only. Do not wrap in markdown or backticks.'
        response = await self.client.chat.completions.create(model=config.model_name, messages=[{'role': 'system', 'content': system_prompt}, {'role': 'user', 'content': augmented_prompt}], temperature=config.temperature, max_tokens=config.max_tokens, response_format={'type': 'json_object'})
        latency = (time.perf_counter() - t0) * 1000.0
        content = response.choices[0].message.content or '{}'
        parsed = schema.model_validate_json(content)
        prompt_tokens = response.usage.prompt_tokens if response.usage else 0
        comp_tokens = response.usage.completion_tokens if response.usage else 0
        telemetry.record_usage(role=role, provider='openrouter', model=config.model_name, prompt_tokens=prompt_tokens, completion_tokens=comp_tokens, latency_ms=latency)
        return parsed

class MockProvider(LLMProvider):

    def __init__(self):
        self.registered_fixtures: dict[Type[BaseModel], Any] = {}

    def register_fixture(self, schema: Type[T], instance: T):
        self.registered_fixtures[schema] = instance

    async def generate_structured(self, schema: Type[T], system_prompt: str, user_prompt: str, role: ModelRole, config: RoleModelConfig) -> T:
        t0 = time.perf_counter()
        if schema in self.registered_fixtures:
            result = self.registered_fixtures[schema]
        else:
            result = self._generate_heuristic_fallback(schema, user_prompt, role)
        latency = (time.perf_counter() - t0) * 1000.0
        telemetry.record_usage(role=role, provider='mock', model='deterministic-heuristic', prompt_tokens=len(user_prompt) // 4, completion_tokens=100, latency_ms=latency)
        return result

    def _generate_heuristic_fallback(self, schema: Type[T], prompt: str, role: ModelRole) -> T:
        fields = schema.model_fields
        data = {}
        for fname, f_info in fields.items():
            ann = str(f_info.annotation)
            if 'list' in ann:
                data[fname] = []
            elif 'str' in ann:
                data[fname] = f'Sample {fname}'
            elif 'float' in ann:
                data[fname] = 0.8
            elif 'int' in ann:
                data[fname] = 10
            elif 'bool' in ann:
                data[fname] = True
            elif 'dict' in ann:
                data[fname] = {}
        try:
            return schema.model_validate(data)
        except Exception:
            return schema.model_construct(**data)

class ModelRouter:

    def __init__(self, settings=llm_settings):
        self.settings = settings
        self._providers: dict[str, LLMProvider] = {}
        self.mock_provider = MockProvider()
        self._init_providers()

    def _init_providers(self):
        self._providers['mock'] = self.mock_provider
        if self.settings.openai_api_key or os.environ.get('OPENAI_API_KEY'):
            try:
                self._providers['openai'] = OpenAIProvider(api_key=self.settings.openai_api_key, base_url=self.settings.openai_base_url)
            except Exception as e:
                logger.warning(f'Failed to initialize OpenAI provider: {e}')
        if self.settings.anthropic_api_key or os.environ.get('ANTHROPIC_API_KEY'):
            try:
                self._providers['anthropic'] = AnthropicProvider(api_key=self.settings.anthropic_api_key)
            except Exception as e:
                logger.warning(f'Failed to initialize Anthropic provider: {e}')
        if self.settings.openrouter_api_key or os.environ.get('OPENROUTER_API_KEY'):
            try:
                self._providers['openrouter'] = OpenRouterProvider(api_key=self.settings.openrouter_api_key, base_url=self.settings.openrouter_base_url)
            except Exception as e:
                logger.warning(f'Failed to initialize OpenRouter provider: {e}')

    def get_provider(self, provider_name: str) -> LLMProvider | None:
        if provider_name in self._providers:
            return self._providers[provider_name]
        return None

    async def generate_structured(self, schema: Type[T], system_prompt: str, user_prompt: str, role: ModelRole, llm_config: dict[str, Any] | None=None) -> T:
        cfg = self.settings.get_role_config(role)
        if llm_config and llm_config.get('provider'):
            user_provider = llm_config.get('provider')
            user_api_key = llm_config.get('api_key')
            user_base_url = llm_config.get('base_url')
            user_model = llm_config.get('model_chat') or cfg.model_name
            if user_provider == 'openai' and user_api_key:
                dynamic_prov = OpenAIProvider(api_key=user_api_key, base_url=user_base_url)
                cfg = RoleModelConfig(provider='openai', model_name=user_model)
                try:
                    return await dynamic_prov.generate_structured(schema, system_prompt, user_prompt, role, cfg)
                except Exception as e:
                    logger.warning(f'Dynamic OpenAI provider failed: {e}. Falling back to default chain.')
            elif user_provider == 'anthropic' and user_api_key:
                dynamic_prov = AnthropicProvider(api_key=user_api_key)
                cfg = RoleModelConfig(provider='anthropic', model_name=user_model)
                try:
                    return await dynamic_prov.generate_structured(schema, system_prompt, user_prompt, role, cfg)
                except Exception as e:
                    logger.warning(f'Dynamic Anthropic provider failed: {e}. Falling back to default chain.')
            elif user_provider == 'openrouter' and user_api_key:
                dynamic_prov = OpenRouterProvider(api_key=user_api_key, base_url=user_base_url or 'https://openrouter.ai/api/v1')
                cfg = RoleModelConfig(provider='openrouter', model_name=user_model)
                try:
                    return await dynamic_prov.generate_structured(schema, system_prompt, user_prompt, role, cfg)
                except Exception as e:
                    logger.warning(f'Dynamic OpenRouter provider failed: {e}. Falling back to default chain.')
        target_provider = cfg.provider
        chain = [target_provider] + [p for p in self.settings.fallback_chain if p != target_provider]
        for prov_name in chain:
            provider = self._providers.get(prov_name)
            if not provider:
                if prov_name == 'mock':
                    provider = self.mock_provider
                else:
                    continue
            try:
                logger.info(f'Routing role {role.value} to provider {prov_name} (model={cfg.model_name})')
                return await provider.generate_structured(schema=schema, system_prompt=system_prompt, user_prompt=user_prompt, role=role, config=cfg)
            except Exception as e:
                logger.warning(f'Provider {prov_name} failed for role {role.value}: {e}. Trying next in fallback chain.')
        logger.info(f'Fallback chain exhausted for role {role.value}. Using MockProvider fallback.')
        return await self.mock_provider.generate_structured(schema=schema, system_prompt=system_prompt, user_prompt=user_prompt, role=role, config=cfg)
router = ModelRouter()
