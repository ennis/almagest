shader {
	glsl_layout float3:0,float3:0,float3:0,float2:0
}

!!GLSL
#version 440
#pragma include <scene.glsl>
#pragma include <scattering.glsl>

layout (std140, binding = 1) uniform SkyParams {
	mat4 modelMatrix;
	float rayleighCoefficient;
	float mieCoefficient;
	float mieDirectionalG;
	float turbidity;
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
#endif // _VERTEX_

#ifdef _FRAGMENT_

layout (binding=0) uniform sampler2D mainTex;

in vec2 tc;
in vec3 wPos;
in vec3 wN;

out vec4 outColor;

void main()
{
	float sunE = sunIntensity(dot(sunDir.xyz, vec3(0.0f, 1.0f, 0.0f)));

	// extinction (absorbtion + out scattering)
	// rayleigh coefficients
	vec3 betaR = totalRayleigh(lambda) * vec3(5.176821E-6, 1.2785348E-5, 2.8530756E-5);;
	// mie coefficients
	vec3 betaM = totalMie(lambda, K, turbidity) * mieCoefficient;

	// optical length
	// cutoff angle at 90 to avoid singularity in next formula.
	float zenithAngle = acos(max(0, dot(up, normalize(wPos - vec3(0, 0, 0)))));
	float sR = rayleighZenithLength / (cos(zenithAngle) + 0.15 * pow(93.885 - ((zenithAngle * 180.0f) / pi), -1.253));
	float sM = mieZenithLength / (cos(zenithAngle) + 0.15 * pow(93.885 - ((zenithAngle * 180.0f) / pi), -1.253));

	// combined extinction factor
	vec3 Fex = exp(-(betaR * sR + betaM * sM));

	// in scattering
	float cosTheta = dot(normalize(wPos - wEye.xyz), sunDir.xyz);
	float rPhase = rayleighPhase(cosTheta);
	vec3 betaRTheta = betaR * rPhase;
	float mPhase = hgPhase(cosTheta, mieDirectionalG);
	vec3 betaMTheta = betaM * mPhase;
	vec3 Lin = sunE * ((betaRTheta + betaMTheta) / (betaR + betaM)) * (1.0f - Fex);

	// nightsky
	vec3 direction = normalize(wPos - wEye.xyz);
	float theta = acos(direction.y); // elevation --> y-axis, [-pi/2, pi/2]
	float phi = atan(direction.z, direction.x); // azimuth --> x-axis [-pi/2, pi/2]
	vec2 uv = vec2(phi, theta) / vec2(2*pi, pi) + vec2(0.5f, 0.0f);
	vec3 L0 = texture(mainTex, uv).rgb * Fex;

	// composition + solar disc
	if (cosTheta > sunAngularDiameterCos)
		L0 += sunE * Fex;

	outColor = vec4(L0 + Lin, 1);
	//fragmentColor1 = logLuminance(fragmentColor0);
}

#endif	// _FRAGMENT_
