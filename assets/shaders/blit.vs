#version 440

layout (std140, binding = 0) uniform BlitData {
	vec2 viewportSize;
};

layout(location=0) in vec2 position;
layout(location=1) in vec2 texcoords;

out vec2 tc;

void main() {
	gl_Position = vec4(((position.x)/viewportSize.x)*2-1,(1-(position.y)/viewportSize.y)*2-1,0, 1);
	tc = texcoords;
}
