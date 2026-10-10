import * as THREE from './vendor/three.module.js';

export class FollowCamera {
  constructor() {
    this.focus = new THREE.Vector3();
    this.direction = new THREE.Vector3();
    this.distance = 6;
    this.ready = false;
  }
  target(position, yaw, pitch, dt) {
    const target = new THREE.Vector3(position.x, position.y + 1.4, position.z);
    if (!this.ready || this.focus.distanceTo(target) > 15) {
      this.focus.copy(target);
      this.distance = 6;
      this.ready = true;
    } else {
      this.focus.x = target.x;
      this.focus.z = target.z;
      // Tread heights should move the rig, not pitch it at each server tick.
      this.focus.y += (target.y - this.focus.y) * (1 - Math.exp(-10 * dt));
    }
    this.direction.set(Math.sin(yaw) * Math.cos(pitch), Math.sin(pitch), Math.cos(yaw) * Math.cos(pitch));
  }
  place(camera, clearance, dt) {
    // Retract immediately to avoid walls; ease outward when an obstruction ends.
    const limit = Math.max(.1, Math.min(6, clearance));
    this.distance = limit < this.distance ? limit : this.distance + (limit - this.distance) * (1 - Math.exp(-3 * dt));
    camera.position.copy(this.focus).addScaledVector(this.direction, this.distance);
    // Position and aim share one smoothed pivot, keeping pitch constant while walking.
    camera.lookAt(this.focus);
  }
}
