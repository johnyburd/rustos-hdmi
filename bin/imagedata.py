from PIL import Image
import sys
import base64

im = Image.open(sys.argv[1]).convert('RGB')
i = [(px[0] << 16) | (px[1] << 8) | px[2] for px in im.getdata()]
#i = b"".join([bytes((0, px[0], px[1], px[2])) for px in im.getdata()])
    #print(bytes(num))
#print(f"pub const IMAGE: &str = \"{base64.b64encode(i).decode()}\";")
#print(f"pub const IMAGE: [u32, {im.size[0] * im.size[1]}] = {i};")
print(f"pub const IMAGE: &'static [u32] = &{i};")
