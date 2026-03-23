// TODO: refactor to common function
function errorToString(e: unknown) {
	if (e instanceof Error) return e.message;
	else return e as string;
}

export default errorToString;
