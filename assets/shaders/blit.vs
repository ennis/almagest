#version 440

layout (std140, binding = 0) uniform SceneData {
	mat4 viewMatrix;
	mat4 projMatrix;
	mat4 viewProjMatrix;	// = projMatrix*viewMatrix
	vec4 lightDir;
	vec4 wEye;	// in world space
	vec2 viewportSize;	// taille de la fenêtre
	vec3 wLightPos;
	vec3 lightColor;
	float lightIntensity;
};

layout(location=0) in vec2 position;
layout(location=1) in vec2 texcoords;

out vec2 tc;

void main() {
	gl_Position = vec4((position.x/viewportSize.x)*2-1,(1-position.y/viewportSize.y)*2-1,0, 1);
	tc = texcoords;
}
