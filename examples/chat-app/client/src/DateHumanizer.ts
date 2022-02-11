function isSameDay(date1: Date, date2: Date) {
  return date1.getFullYear() === date2.getFullYear() &&
    date1.getMonth() === date2.getMonth() &&
    date1.getDate() === date2.getDate();
}

export default function humanize(date: Date): string {
  // If today, show date as HH:MM
  let now = new Date();
  if (isSameDay(now, date)) {
    return date.toLocaleTimeString("en-GB", { hour: "2-digit", minute: "2-digit" });
  }

  // If yesterday, show "yesterday"
  let yesterday = new Date();
  yesterday.setDate(yesterday.getDate() - 1);
  if (isSameDay(yesterday, date)) {
    return "yesterday";
  }

  // If in the last week, show day of the week
  let lastWeek = new Date();
  lastWeek.setDate(lastWeek.getDate() - 7);
  if (lastWeek.getTime() < date.getTime()) {
    return [
      "Sunday",
      "Monday",
      "Tuesday",
      "Wednesday",
      "Thursday",
      "Friday",
      "Saturday"
    ][date.getDay()];
  }

  // Otherwise, return DD/MM/YY
  return date.toLocaleDateString("en-GB", { day: "2-digit", month: "2-digit", year: "2-digit" });
}