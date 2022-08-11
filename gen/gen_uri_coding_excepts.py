import string

from gen import base

uri: [str] = ["false"] * 128
uri_comp: [str] = ["false"] * 128

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


def run():
	distfn = base.dist("../src/h2tp/utils/uricoding_excepts.rs")
	base.render(distfn, "uri_encoding_excepts.jinja2", uri=f"[{', '.join(uri)}]", uri_comp=f"[{', '.join(uri_comp)}]")
	base.log("done")
