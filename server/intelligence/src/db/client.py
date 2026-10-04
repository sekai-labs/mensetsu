import httpx
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    surreal_url: str = 'http://127.0.0.1:8000'
    surreal_endpoint: str | None = None
    surreal_user: str = 'root'
    surreal_pass: str = 'root'
    surreal_ns: str = 'mensetsu'
    surreal_db: str = 'production'
    temporal_host: str = '127.0.0.1:7233'
    task_queue: str = 'mensetsu-research-queue'

    def get_surreal_url(self) -> str:
        return self.surreal_endpoint or self.surreal_url

settings = Settings()

class SurrealDirectClient:

    def __init__(self, settings: Settings=settings):
        self.base_url = settings.get_surreal_url().rstrip('/')
        self.auth = (settings.surreal_user, settings.surreal_pass)
        self.headers = {'surreal-ns': settings.surreal_ns, 'surreal-db': settings.surreal_db, 'Accept': 'application/json'}
    async def execute_query(self, query: str) -> list[dict]:
        async with httpx.AsyncClient(timeout=10.0) as client:
            resp = await client.post(f'{self.base_url}/sql', auth=self.auth, headers=self.headers, content=query.encode('utf-8'))
            resp.raise_for_status()
            return resp.json()
