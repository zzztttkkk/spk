import string

from gen import base

uri: [str] = ["false"] * 256
uri_comp: [str] = ["false"] * 256
hex: [int] = [16] * 256

for c in string.ascii_letters + string.digits:
	uri[ord(c)] = "true"
	uri_comp[ord(c)] = "true"

# https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/encodeURI
for c in "; , / ? : @ & = + $ - _ . ! ~ * ' ( ) #":
	if c == " ":
		continue
	uri[ord(c)] = "true"

# https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/encodeURIComponent
for c in "- _ . ! ~ * ' ( )":
	if c == " ":
		continue
	uri_comp[ord(c)] = "true"

for i in range(256):
	if ord("0") <= i <= ord("9"):
		hex[i] = i - ord("0")
	elif ord("a") <= i <= ord("f"):
		hex[i] = i - ord("a") + 10
	elif ord("A") <= i <= ord("F"):
		hex[i] = i - ord("A") + 10

def run():
	distfn = base.dist("../src/h2tp/utils/uricoding_excepts.rs")
	base.render(distfn, "uri_encoding_excepts.jinja2", uri=f"[{', '.join(uri)}]", uri_comp=f"[{', '.join(uri_comp)}]", hex=hex)
	base.log("done")
