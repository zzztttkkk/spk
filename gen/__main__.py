import inspect
import os
import threading
import typing

cwd = os.path.abspath(os.path.dirname(os.path.abspath(__file__)))
os.chdir(cwd)
os.system("pip install -r ./requirements.txt")

funcs: [typing.Callable] = []

for fn in os.listdir(cwd):
	if fn.startswith("gen_") and fn.endswith(".py"):
		name = fn[:-3]
		module = getattr(__import__(f"gen", fromlist=[name]), name)
		func = getattr(module, "run", None)
		if inspect.isfunction(func):
			funcs.append(func)

ts = []
for fn in funcs:
	t = threading.Thread(target=fn)
	t.setDaemon(True)
	t.start()
	ts.append(t)

for t in ts:
	t.join()
