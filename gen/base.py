import inspect
import os
import pathlib
import threading
import typing
import jinja2

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


def render(fn: str, tpl: str, **kwargs):
	env = jinja2.Environment(loader=jinja2.FileSystemLoader("./"))
	tpl = env.get_template(tpl)

	with open(fn, "w+") as f:
		f.truncate(0)
		f.write(tpl.render(**kwargs))
