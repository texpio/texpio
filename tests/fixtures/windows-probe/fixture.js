const field = document.getElementById('probe-text')
document.getElementById('reset-text').addEventListener('click', () => {
  field.disabled = false
  field.readOnly = false
  field.value = 'TEXPIO_P0_BEGIN\n\nTEXPIO_P0_END'
  field.focus()
  field.setSelectionRange(16, 16)
})
document.getElementById('readonly-text').addEventListener('click', () => {
  field.readOnly = !field.readOnly
})
document.getElementById('disable-text').addEventListener('click', () => {
  field.disabled = !field.disabled
})
