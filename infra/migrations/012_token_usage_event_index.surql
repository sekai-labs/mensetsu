USE NS mensetsu DB production;

DEFINE INDEX IF NOT EXISTS idx_execution_event_type ON TABLE execution_event COLUMNS type;
