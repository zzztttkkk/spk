import os
import re
import time

import requests
from bs4 import BeautifulSoup
import jinja2

from gen import base


def run():
	distfn = base.dist("../src/h2tp/status_code.rs")
	try:
		status = os.stat(distfn)
		if time.time() - status.st_mtime < 86400 * 15:
			base.log("done")
			return
	except FileNotFoundError:
		...

	html = requests.get("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status").text
	soup = BeautifulSoup(html, "html.parser")

	items = []

	for ele in soup.select("dt a code"):
		contents = ele.contents
		if not contents:
			continue
		content: str = contents[0].text
		idx = content.find(" ")
		if idx < 0:
			continue

		try:
			num = int(content[:idx])
		except ValueError:
			continue

		msg = content[idx + 1:]
		if msg[0].islower():
			continue

		name = re.sub(r"[\s\W]+", "", msg)
		if name == "Imateapot":
			name = "ImATeapot"

		items.append({"name": name, "num": num, "msg": msg})

	if not items:
		base.log("empty items")
		return

	env = jinja2.Environment(loader=jinja2.FileSystemLoader("./"))
	tpl = env.get_template("status_code_rs.jinja2")

	with open(distfn, "w+") as f:
		f.truncate(0)
		f.write(tpl.render(items=items))

	base.log("done")
