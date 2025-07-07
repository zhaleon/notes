// This file makes TypeScript aware of the $lib alias
declare module '$lib/*' {
  const value: any;
  export default value;
}
