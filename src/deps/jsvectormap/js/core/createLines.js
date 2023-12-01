import { merge, getLineUid } from '../util'
import Line from '../components/line'

export default function createLines(lines, markers, isRecentlyCreated = false) {
  let point1 = false, point2 = false

  // Create group for holding lines
  // we're checking if `linesGroup` exists or not becuase we may add lines
  // after the map has loaded so we will append the futured lines to this group as well.
  this.linesGroup = this.linesGroup || this.canvas.createGroup('jvm-lines-group')
  
  for (let index in lines) {
    const config = lines[index]

    for (let mindex in markers) {
      const markerConfig = isRecentlyCreated ? markers[mindex].config : markers[mindex]

      if (markerConfig.name === config.from) {
        point1 = this.getMarkerPosition(markerConfig)
      }

      if (markerConfig.name === config.to) {
        point2 = this.getMarkerPosition(markerConfig)
      }
    }

    if (point1 !== false && point2 !== false) {
      // Register lines with unique keys
      this._lines[getLineUid(config.from, config.to)] = new Line({
        index: index,
        map: this,
        // Merge the default `lineStyle` object with the custom `line` config style
        style: merge({ initial: this.params.lineStyle }, { initial: config.style || {} }, true),
        x1: point1.x,
        y1: point1.y,
        x2: point2.x,
        y2: point2.y,
        group: this.linesGroup,
        config
      })
    }
  }
}