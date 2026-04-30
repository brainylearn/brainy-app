import { useCallback, useState } from "react";
import useApi from "./useApi";

/** A hook that lets you combine an error message with the useApi hook into
 * a single state so that you don't have to manage to error messages. */
export default function useApiWithCustomError() {
	const [customErrorMessage, setCustomErrorMessage] = useState<string | null>(
		null,
	);
	const {
		isSendingRequest,
		errorMessage: apiErrorMessage,
		clearErrorMessage: clearApiErrorMessage,
		callApi,
	} = useApi({
		onFinally: () => setCustomErrorMessage(null),
	});

	const clearErrorMessage = useCallback(() => {
		clearApiErrorMessage();
		setCustomErrorMessage(null);
	}, [clearApiErrorMessage]);

	return {
		isSendingRequest,
		errorMessage: apiErrorMessage ?? customErrorMessage,
		callApi,
		clearErrorMessage,
		setCustomErrorMessage,
	};
}
