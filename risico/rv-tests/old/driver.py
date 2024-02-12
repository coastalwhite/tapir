#! /usr/bin/env python

import sys

class ProcessCommandTag:
    INIT = 0
    ACK = 1
    SIZE = 2
    GET = 8
    SET = 9
    RESULT = 10

class ProcessCommand:
    CMD_ARGS = {
        ProcessCommandTag.INIT: [],
        ProcessCommandTag.ACK: [],
        ProcessCommandTag.SIZE: [4],
        ProcessCommandTag.GET: [4],
        ProcessCommandTag.SET: [4,4],
        ProcessCommandTag.RESULT: [4],
    }

    def __init__(self, tag: int, args: list[int] = []) -> None:
        assert len(args) == len(ProcessCommand.CMD_ARGS[tag])

        self.tag = tag
        self.args = args

    def serialize(self):
        sys.stdout.buffer.write(self.tag.to_bytes(length = 2, byteorder="big"))

        for i, arg in enumerate(self.args):
            bs = arg.to_bytes(length = self.CMD_ARGS[self.tag][i], byteorder="big")
            sys.stdout.buffer.write(bs)

        sys.stdout.flush()


def deserialize() -> ProcessCommand:
    tag_buffer = sys.stdin.buffer.read(2)
    tag = int.from_bytes(tag_buffer, byteorder="big")

    args = []
    for arg_size in ProcessCommand.CMD_ARGS[tag]:
        arg_buffer = sys.stdin.buffer.read(arg_size)
        args.append(int.from_bytes(arg_buffer, byteorder="big"))

    return ProcessCommand(tag, args)

def ack() -> None:
    ProcessCommand(ProcessCommandTag.ACK, []).serialize()

init = deserialize()
assert init.tag == ProcessCommandTag.INIT
ack()

size = deserialize()
assert size.tag == ProcessCommandTag.SIZE
ack()

while True:
    cmd = deserialize()

    match cmd.tag:
        case ProcessCommandTag.GET:
            ProcessCommand(ProcessCommandTag.RESULT, [69]).serialize()
        case ProcessCommandTag.SET:
            ProcessCommand(ProcessCommandTag.RESULT, [1337]).serialize()
        case _:
            exit(1)
