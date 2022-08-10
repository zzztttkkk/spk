import inspect
import os
import pathlib
import threading
import typing

cwd = pathlib.Path(os.path.abspath(__file__)).parent.absolute()


def dist(name: str) -> str:
	return os.path.abspath(os.path.join(cwd, name))


lock = threading.Lock()


def log(msg: str, *args: typing.Any):
	with lock:
		filename = ""
		for f in inspect.getouterframes(inspect.currentframe()):
			filename = os.path.basename(f.filename)
			if filename.startswith("gen") and filename.endswith(".py"):
				break
			filename = ""
		print(f"{filename}: {msg.format(*args)}")
