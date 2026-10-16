from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class SubscribeRunMessage(_message.Message):
    __slots__ = ("run_id", "after_seq")
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    AFTER_SEQ_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    after_seq: int
    def __init__(self, run_id: _Optional[str] = ..., after_seq: _Optional[int] = ...) -> None: ...

class UnsubscribeRunMessage(_message.Message):
    __slots__ = ("run_id",)
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    def __init__(self, run_id: _Optional[str] = ...) -> None: ...

class AckMessage(_message.Message):
    __slots__ = ("run_id", "ack_seq")
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    ACK_SEQ_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    ack_seq: int
    def __init__(self, run_id: _Optional[str] = ..., ack_seq: _Optional[int] = ...) -> None: ...

class PingMessage(_message.Message):
    __slots__ = ("timestamp",)
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    timestamp: int
    def __init__(self, timestamp: _Optional[int] = ...) -> None: ...

class EventStreamClientMessage(_message.Message):
    __slots__ = ("subscribe", "unsubscribe", "ack", "ping")
    SUBSCRIBE_FIELD_NUMBER: _ClassVar[int]
    UNSUBSCRIBE_FIELD_NUMBER: _ClassVar[int]
    ACK_FIELD_NUMBER: _ClassVar[int]
    PING_FIELD_NUMBER: _ClassVar[int]
    subscribe: SubscribeRunMessage
    unsubscribe: UnsubscribeRunMessage
    ack: AckMessage
    ping: PingMessage
    def __init__(self, subscribe: _Optional[_Union[SubscribeRunMessage, _Mapping]] = ..., unsubscribe: _Optional[_Union[UnsubscribeRunMessage, _Mapping]] = ..., ack: _Optional[_Union[AckMessage, _Mapping]] = ..., ping: _Optional[_Union[PingMessage, _Mapping]] = ...) -> None: ...

class ExecutionEventFrame(_message.Message):
    __slots__ = ("seq", "run_id", "event_type", "task_id", "execution_id", "payload_json", "timestamp", "is_terminal")
    SEQ_FIELD_NUMBER: _ClassVar[int]
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    EVENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    TASK_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_JSON_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    IS_TERMINAL_FIELD_NUMBER: _ClassVar[int]
    seq: int
    run_id: str
    event_type: str
    task_id: str
    execution_id: str
    payload_json: str
    timestamp: str
    is_terminal: bool
    def __init__(self, seq: _Optional[int] = ..., run_id: _Optional[str] = ..., event_type: _Optional[str] = ..., task_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., payload_json: _Optional[str] = ..., timestamp: _Optional[str] = ..., is_terminal: _Optional[bool] = ...) -> None: ...
