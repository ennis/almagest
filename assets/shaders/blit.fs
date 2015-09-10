#version 440

layout (binding=0) uniform sampler2D blitTex;

in vec2 tc;
out vec4 color;

void main() {
	color = texture(blitTex, tc);
}
