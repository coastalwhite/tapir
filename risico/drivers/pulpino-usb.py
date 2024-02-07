#! /usr/bin/env python

import sys

class ProcessCommandTag:
    INIT = 0
    ACK = 1
    SIZE = 2
    GET = 8
    SET = 9
    RESULT = 10
    ERROR = 256

class ProcessCommand:
    CMD_ARGS = {
        ProcessCommandTag.INIT: [],
        ProcessCommandTag.ACK: [],
        ProcessCommandTag.SIZE: [4],
        ProcessCommandTag.GET: [4],
        ProcessCommandTag.SET: [4,4],
        ProcessCommandTag.RESULT: [4],
        ProcessCommandTag.ERROR: [4],
    }

    def __init__(self, tag: int, args: list[int] | list[str] = []) -> None:
        assert len(args) == len(ProcessCommand.CMD_ARGS[tag])

        self.tag = tag
        self.args = args

    def serialize(self):
        sys.stdout.buffer.write(self.tag.to_bytes(length = 2, byteorder="big"))

        if self.tag == ProcessCommandTag.ERROR:
            assert isinstance(self.args[0], str)
            size = len(self.args[0]).to_bytes(length = 4, byteorder="big")
            sys.stdout.buffer.write(size)
            sys.stdout.buffer.write(self.args[0].encode(encoding="utf-8"))
        else:
            for i, arg in enumerate(self.args):
                assert isinstance(arg, int)
                bs = arg.to_bytes(length = self.CMD_ARGS[self.tag][i], byteorder="big")
                sys.stdout.buffer.write(bs)

        sys.stdout.flush()


def deserialize() -> ProcessCommand:
    tag_buffer = sys.stdin.buffer.read(2)
    tag = int.from_bytes(tag_buffer, byteorder="big")

    args = []

    # TODO: Deserialize Error
    assert tag != ProcessCommandTag.ERROR

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
assert size.args[0] == 12
ack()

gpio_out = 0
state_ctr = 0

STATE = 1

with open("usb.log", "w") as f:
    f.write('\n')

while True:
    cmd = deserialize()
    ack()

    state_ctr += 1
    if state_ctr == 100:
        STATE += 1
        STATE %= 4
        state_ctr = 0

    with open("usb.log", "a") as f:
        f.write('RECEIVED COMMAND')
        f.write('\n')
        f.write(str(cmd.tag))
        f.write('\n')
        f.write(str(cmd.args))
        f.write('\n')

    match cmd.tag:
        case ProcessCommandTag.GET | ProcessCommandTag.SET:
            addr = cmd.args[0]
            assert isinstance(addr, int)

            if addr >= 0 and addr < 4:
                ProcessCommand(ProcessCommandTag.ERROR, ["GPIO_ADDR used"]).serialize()
            elif addr >= 4 and addr < 8:
                # GPIO_IN
                if cmd.tag == ProcessCommandTag.GET:
                    ext_read_flicker = 0
                    ext_write_flicker = 0
                    ext_data = 0

                    if STATE == 1 or STATE == 2:
                        ext_read_flicker = 1
                        ext_write_flicker = 1

                    gpio_in = (ext_read_flicker << 9) | (ext_write_flicker << 8) | ext_data

                    with open("usb.log", "a") as f:
                        f.write(f'GPIO_IN: {hex(gpio_in)}')
                        f.write('\n')

                    ProcessCommand(ProcessCommandTag.RESULT, [gpio_in]).serialize()
                else:
                    ProcessCommand(ProcessCommandTag.ERROR, ["Setting GPIO_IN"]).serialize()
            elif addr >= 8 and addr < 12:
                # GPIO_OUT
                if cmd.tag == ProcessCommandTag.SET:
                    gpio_out = int(cmd.args[1])
                    
                    with open("usb.log", "a") as f:
                        f.write(hex(gpio_out))
                        f.write('\n')
                else:
                    ProcessCommand(ProcessCommandTag.RESULT, [gpio_out]).serialize()
            else:
                with open("usb.log", "a") as f:
                    f.write('OUT OF RANGE')
                    f.write('\n')
                ProcessCommand(ProcessCommandTag.ERROR, ["Address out of range"]).serialize()
        case _:
            ProcessCommand(ProcessCommandTag.ERROR, ["Invalid command"]).serialize()