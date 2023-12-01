export default function repositionLines() {
  let point1 = false, point2 = false

  for (let index in this._lines) {
    for (let mindex in this._markers) {
      const marker = this._markers[mindex]

      if (marker.config.name === this._lines[index].config.from) {
        point1 = this.getMarkerPosition(marker.config)
      }

      if (marker.config.name === this._lines[index].config.to) {
        point2 = this.getMarkerPosition(marker.config)
      }
    }

    if (point1 !== false && point2 !== false) {
      this._lines[index].setStyle({
        x1: point1.x,
        y1: point1.y,
        x2: point2.x,
        y2: point2.y,
      })
    }
  }
}