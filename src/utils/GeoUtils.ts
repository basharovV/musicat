import * as THREE from "three";
import { Color } from "three";
import type { Song, SongsByCountry } from "../App";
import { get } from "svelte/store";
import { userSettings } from "../data/store";
import { countries } from "../lib/data/CountryCodes";

// Convert XYZ coordinates to latitude and longitude
export function xyzToLatLng(point: THREE.Vector3): {
    lat: number;
    lng: number;
} {
    const normalized = point.clone().normalize();

    // Calculate latitude (phi)
    const lat = 90 - THREE.MathUtils.radToDeg(Math.acos(normalized.y));

    // Calculate longitude (theta)
    let lng =
        THREE.MathUtils.radToDeg(Math.atan2(normalized.z, -normalized.x)) - 180;

    // Normalize longitude to [-180, 180]
    if (lng < -180) lng += 360;
    if (lng > 180) lng -= 360;

    return { lat, lng };
}

export function triangulatePolygon(
    outer: number[][],
    holes: number[][][],
    radius: number,
): THREE.BufferGeometry {
    const vertices2D: THREE.Vector2[] = [];
    const holeIndices: number[] = [];

    outer.forEach(([lng, lat]) => {
        vertices2D.push(new THREE.Vector2(lng, lat));
    });

    holes.forEach((ring) => {
        holeIndices.push(vertices2D.length);
        ring.forEach(([lng, lat]) => {
            vertices2D.push(new THREE.Vector2(lng, lat));
        });
    });

    const indices = THREE.ShapeUtils.triangulateShape(
        vertices2D.slice(0, outer.length),
        holes.map((r) => r.map(([lng, lat]) => new THREE.Vector2(lng, lat))),
    );

    const positions: number[] = [];

    vertices2D.forEach((v) => {
        const p = latLngToSphere(v.y, v.x, radius);
        positions.push(p.x, p.y, p.z);
    });

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute(
        "position",
        new THREE.Float32BufferAttribute(positions, 3),
    );
    geometry.setIndex(indices.flat());

    geometry.computeVertexNormals();
    geometry.computeBoundingSphere();

    return geometry;
}

export function latLngToSphere(lat: number, lng: number, radius: number) {
    const phi = THREE.MathUtils.degToRad(90 - lat);
    const theta = THREE.MathUtils.degToRad(lng + 180);

    return new THREE.Vector3(
        -radius * Math.sin(phi) * Math.cos(theta),
        radius * Math.cos(phi),
        radius * Math.sin(phi) * Math.sin(theta),
    );
}

export function sphericalInterpolation(
    pointA: THREE.Vector3,
    pointB: THREE.Vector3,
    t: number,
    distance: number,
): THREE.Vector3 {
    // Normalize the input points
    const v0 = pointA.clone().normalize();
    const v1 = pointB.clone().normalize();

    // Compute the angle between the two vectors
    const dot = v0.dot(v1);
    const omega = Math.acos(dot);

    // Perform spherical linear interpolation
    const sinOmega = Math.sin(omega);
    const scale0 = Math.sin((1 - t) * omega) / sinOmega;
    const scale1 = Math.sin(t * omega) / sinOmega;

    // Interpolated vector on the unit sphere
    const interpolated = v0
        .multiplyScalar(scale0)
        .add(v1.multiplyScalar(scale1));

    // Scale to the desired distance from the center
    return interpolated.multiplyScalar(distance);
}

export function getCountryColor(
    countryCode: string,
    songsByCountry: SongsByCountry,
    fromColor: Color | string | number,
    toColor: Color | string | number,
): Color | null {
    const entries = Object.entries(songsByCountry);

    if (entries.length === 0) {
        return null;
    }

    // Compute min/max counts for normalization
    let min = Infinity;
    let max = -Infinity;

    for (const [, value] of entries) {
        const count = value.data.length;
        min = Math.min(min, count);
        max = Math.max(max, count);
    }

    let countryData;
    if (get(userSettings).beetsDbLocation) {
        countryData = songsByCountry[countryCode];
    } else {
        countryData = songsByCountry[countries[countryCode]];
    }
    if (!countryData) {
        return null;
    }
    const count = countryData?.data.length ?? 0;

    // Avoid division by zero when all countries have same count
    const t = max === min ? 0 : (count - min) / (max - min);

    return new Color(fromColor).lerp(new Color(toColor), t);
}
