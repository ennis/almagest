shader {
	glsl_layout float3:0,float3:0,float3:0,float2:0
}

!!GLSL
#version 440
#pragma include <scene.glsl>

layout (std140, binding = 1) uniform ObjectData {
	mat4 modelMatrix;
};

#ifdef _VERTEX_
layout(location=0) in vec3 position;
layout(location=1) in vec3 normal;
layout(location=2) in vec3 tangent;
layout(location=3) in vec2 texcoords;

out vec3 wN;
out vec3 vN;

void main() {
	vec4 wPos_tmp = modelMatrix * vec4(position, 1.0);
	vec4 wN_tmp = modelMatrix * vec4(normal, 0.0);
	wN = wN_tmp.xyz;
	gl_Position = wPosToClipSpace(wPos_tmp);
	vN = wNToCameraSpace(wN);
}
#endif // _VERTEX_

#ifdef _FRAGMENT_
in vec3 vN;
in vec3 wN;
out vec4 color;
void main() {
	color = vec4(vN, 1.0);
}
#endif // _