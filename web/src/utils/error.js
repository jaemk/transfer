
export const logerr = (err) => {
  console.log(err)
}

export const InvalidHexException = (s) => {
  this.message = s
  this.name = 'InvalidHexException'
}

export const InvalidOptionException = (s) => {
  this.message = s
  this.name = 'InvalidOptionException'
}

