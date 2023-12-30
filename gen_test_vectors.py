#!/usr/bin/env python3

from nxt.telegram import Opcode, Telegram


def out(label, pkt):
	print("const", label, ": &[u8] = &", list(pkt.pkt.getvalue()), ";")

def main():
	get_bat = Telegram(Opcode.DIRECT_GET_BATT_LVL)
	out("BATT_LEVEL", get_bat)

	brick_name = Telegram(Opcode.SYSTEM_SETBRICKNAME)
	brick_name.add_string(15, "test")
	out("BRICK_NAME", brick_name)


if __name__ == "__main__":
	main()

