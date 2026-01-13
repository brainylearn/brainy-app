import styles from "./styles.module.css";
import Dialog from "../../../components/Dialog/Dialog";
import Form, {
	FormButtons,
	FormHeader,
	FormRows,
} from "../../../components/Form/Form";
import { mdiDeleteOutline, mdiPlusBoxOutline, mdiTuneVariant } from "@mdi/js";
import Icon from "@mdi/react";
import { useCallback, useEffect, useState } from "react";
import errorToString from "../../../utils/errorToString";
import Alert from "../../../components/Alert/Alert";
import FsrsProfile from "../../../types/backend/entity/fsrsProfile";
import {
	createProfile,
	getAllFsrsProfiles,
	getFileFsrsProfile,
	getFolderFsrsProfile,
	getParentProfileForFile,
	getParentProfileForFolder,
	updateProfile,
} from "../../../api/fsrsApi";
import { ROOT_FOLDER_ID } from "../../../config/constants";

interface Props {
	id: string;
	isFolder: boolean;
	onClose: () => void;
}

// TODO: unit test
export default function FsrsDialog({ id, isFolder, onClose }: Props) {
	const [allFsrsProfiles, setAllFsrsProfiles] = useState<FsrsProfile[]>([]);
	const [errorMessage, setErrorMessage] = useState("");
	const [chosenProfile, setChosenProfile] = useState("");
	const [profileState, setProfileState] = useState<FsrsProfile | null>();

	const isRoot = id === ROOT_FOLDER_ID;

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		// TODO: update chosen profile for file
		// TODO: error handling

		await updateProfile(profileState!);
		onClose();
	};

	const executeRequest = useCallback(
		async (cb: () => Promise<void>): Promise<void> => {
			try {
				await cb();
			} catch (e) {
				console.error(e);
				setErrorMessage(errorToString(e));
			}
		},
		[],
	);

	useEffect(() => {
		void (async () => {
			setAllFsrsProfiles(await getAllFsrsProfiles());
			const itemProfile = isFolder
				? await getFolderFsrsProfile(id)
				: await getFileFsrsProfile(id);

			setChosenProfile(itemProfile.id);
			setProfileState(itemProfile);
		})();
	}, [executeRequest, isFolder, id]);

	const handleChangeProfile = async (newValue: string) => {
		if (newValue === "inherit") {
			const itemProfile = isFolder
				? await getParentProfileForFolder(id)
				: await getParentProfileForFile(id);
			setProfileState(itemProfile);
		} else {
			const itemProfile = allFsrsProfiles.find(p => p.id === newValue);
			setProfileState(itemProfile);
		}
		setChosenProfile(newValue);
	};

	const handleCloneProfile = async () => {
		if (!profileState) return;

		const profile = await createProfile({
			name: profileState.name + " clone",
			maximumInterval: profileState.maximumInterval,
			requestRetention: profileState.requestRetention,
			weights: profileState.weights,
		});

		setAllFsrsProfiles(await getAllFsrsProfiles());
		setProfileState(profile);
		setChosenProfile(profile.id);
	};

	return (
		<Dialog onHide={onClose} focusTrap className={styles.fsrsDialog}>
			<Form onSubmit={e => void handleSubmit(e)}>
				<FormHeader icon={mdiTuneVariant} title="FSRS Profile" />

				{profileState && (
					<FormRows
						rows={[
							{
								label: "Profile",
								labelHtmlFor: "profile",
								children: (
									<div className={styles.chooseProfileRow}>
										<select
											id="profile"
											value={chosenProfile}
											onChange={e =>
												void handleChangeProfile(
													e.target.value,
												)
											}
											autoFocus>
											{!isRoot && (
												<option value="inherit">
													Inherit from parent
												</option>
											)}
											{allFsrsProfiles.map(profile => (
												<option
													value={profile.id}
													key={profile.id}>
													{profile.name}
												</option>
											))}
										</select>
										<button
											className="transparent"
											type="button"
											title="Clone profile"
											onClick={() =>
												void handleCloneProfile()
											}>
											<Icon
												path={mdiPlusBoxOutline}
												size={1}
											/>
										</button>
										<button
											className="red"
											type="button"
											title="Delete profile">
											<Icon
												path={mdiDeleteOutline}
												size={1}
											/>
										</button>
									</div>
								),
							},
							{
								label: "Name",
								labelHtmlFor: "name",
								children: (
									<input
										id="name"
										type="text"
										minLength={1}
										value={profileState.name}
										onChange={e =>
											setProfileState({
												...profileState,
												name: e.target.value,
											})
										}
										readOnly={chosenProfile === "inherit"}
										required
									/>
								),
							},
							{
								label: "Request retention",
								labelHtmlFor: "request-retention",
								children: (
									<input
										id="request-retention"
										type="number"
										step="any"
										readOnly={chosenProfile === "inherit"}
										value={profileState.requestRetention}
										onChange={e =>
											setProfileState({
												...profileState,
												requestRetention: Number(
													e.target.value,
												),
											})
										}
										required
									/>
								),
							},
							{
								label: "Maximum interval",
								labelHtmlFor: "maximum-interval",
								children: (
									<input
										id="maximum-interval"
										type="number"
										step="any"
										readOnly={chosenProfile === "inherit"}
										value={profileState.maximumInterval}
										onChange={e =>
											setProfileState({
												...profileState,
												maximumInterval: Number(
													e.target.value,
												),
											})
										}
										required
									/>
								),
							},
							{
								label: "Weights",
								labelHtmlFor: "weights",
								children: (
									<textarea
										id="weights"
										placeholder="TODO:"
										rows={3}
										readOnly={chosenProfile === "inherit"}
										value={profileState.weights.join(" ")}
										// TODO: better validation
										onChange={e =>
											setProfileState({
												...profileState,
												weights: e.target.value
													.split(" ")
													.map(w => Number(w)),
											})
										}
										required
									/>
								),
							},
						]}
					/>
				)}

				{errorMessage && (
					<Alert type="error" onClose={() => setErrorMessage("")}>
						<p>{errorMessage}</p>
					</Alert>
				)}

				<FormButtons onClose={onClose} submitText="Save" />
			</Form>
		</Dialog>
	);
}
