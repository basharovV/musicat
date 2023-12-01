import BaseComponent from './base'

/**
 * ------------------------------------------------------------------------
 * Class Definition
 * ------------------------------------------------------------------------
 */
class Line extends BaseComponent {
  constructor({ index, map, style, x1, y1, x2, y2, group, config }) {
    super()

    this.config = config
    this.shape = map.canvas.createLine({ x1, y1, x2, y2, dataIndex: index }, style, group)
    this.shape.addClass('jvm-line')
  }

  setStyle(property, value) {
    this.shape.setStyle(property, value)
  }
}

export default Line