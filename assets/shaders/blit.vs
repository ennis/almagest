shader {
	glsl_layout float2:0,float2:0
}

!!GLSL
#version 440

layout (std140, binding = 0) uniform BlitData {
	vec2 viewportSize;
};

#ifdef _VERTEX_
layout(location=0) in vec2 position;
layout(location=1) in vec2 texcoords;
out vec2 tc;
void main() {
	gl_Position = vec4(((position.x)/viewportSize.x)*2-1,(1-(position.y)/viewportSize.y)*2-1,0, 1);
	tc = texcoords;
}
#endif

#ifdef _FRAGMENT_
layout (binding=0) uniform sampler2D blitTex;
in vec2 tc;
out vec4 color;
void main() {
	color = texture(blitTex, tc);
}
#endif
