shader {}
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
out vec2 tc;
out vec3 wPos;
out vec3 wN;
void main() {
	vec4 wPos_tmp = modelMatrix * vec4(position, 1.0);
	// TODO normal matrix
	vec4 wN_tmp = modelMatrix * vec4(normal, 0.0);
	gl_Position = wPosToClipSpace(wPos_tmp);
	tc = texcoords;
	wPos = wPos_tmp.xyz;
	wN = wN_tmp.xyz;
}
#endif

#ifdef _FRAGMENT_
#ifndef SHADOW_MAP
layout (binding=0) uniform sampler2D mainTex;
in vec2 tc;
in vec3 wPos;
in vec3 wN;
out vec4 color;
void main() {
	//color = vec4(tc, 0.0, 0.0);
	color = texture(mainTex, tc);
}
#endif 	// !SHADOW_MAP
#ifdef SHADOW_MAP
out float fragDepth;
void main()
{
	fragDepth = gl_FragCoord.z;
}
#endif	// SHADOW_MAP
#endif	// _FRAGMENT_
