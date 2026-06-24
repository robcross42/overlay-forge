export function timestampLabel(date: Date) {
  const year = date.getFullYear();
  const month = padDatePart(date.getMonth() + 1);
  const day = padDatePart(date.getDate());
  const hours = padDatePart(date.getHours());
  const minutes = padDatePart(date.getMinutes());
  const seconds = padDatePart(date.getSeconds());

  return `${year}${month}${day}_${hours}${minutes}${seconds}`;
}

function padDatePart(value: number) {
  return value.toString().padStart(2, "0");
}
