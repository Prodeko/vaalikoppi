var SITE_ROOT_PATH = "/vaalikoppi/";
var SOUND_STATE = 0;
var currentVotingId = -1;
var votesGiven = 0;

// Helper functions
function getCookie(name) {
  let cookieValue = null;
  if (document.cookie && document.cookie !== "") {
    const cookies = document.cookie.split(";");
    for (let i = 0; i < cookies.length; i++) {
      const cookie = cookies[i].trim();
      if (cookie.substring(0, name.length + 1) === name + "=") {
        cookieValue = decodeURIComponent(cookie.substring(name.length + 1));
        break;
      }
    }
  }
  return cookieValue;
}

function callApi(url, method, body) {
  return fetch(url, {
    method: method,
    headers: {
      "Content-Type": "application/json",
      "X-CSRFToken": getCookie("csrftoken"),
      "X-Requested-With": "XMLHttpRequest",
    },
    mode: "same-origin",
    body: JSON.stringify(body),
  });
}

function getVotingForm(votingId) {
  return document.getElementById(`voting-form-${votingId}`);
}

function genUserNotifObj(color, displayLength) {
  return { color: color, displayLength: displayLength };
}

const USER_NOTIFICATION = {
  WARNING: genUserNotifObj("red", 6000),
  CONFIRMATION: genUserNotifObj("green", 6000),
  ALERT: genUserNotifObj("orange", 6000),
};

function showUserNotification(notifType, message) {
  M.toast({
    html: message,
    classes: notifType.color,
    displayLength: notifType.displayLength,
  });
}

// Used to sort candidates by position in ranked choice
// election confirmation modal
function compareChosenCandidates(a, b) {
  if (a.position.charAt(0) == "-") {
    return 1;
  }
  if (b.position.charAt(0) == "-") {
    return -1;
  }
  if (a.position < b.position) {
    return -1;
  }
  if (a.position > b.position) {
    return 1;
  }
  return 0;
}

// Voting
function getChosenCandidates(isRankedChoice, votingId) {
  const form = getVotingForm(votingId);
  const chosenCandidates = Array.from(
    form.querySelectorAll(
      isRankedChoice ? ".voting-order" : "input[name=candidate]:checked"
    )
  ).map((input) => {
    const candidateId = input.getAttribute("value");
    const nameNode = form.querySelector(
      `label[for=candidate-v-${votingId}-${candidateId}]`
    );
    const labelText = nameNode.childNodes[0].textContent;
    const labelRankedPosition = isRankedChoice
      ? nameNode.previousElementSibling.innerHTML
      : -1;
    return { id: candidateId, name: labelText, position: labelRankedPosition };
  });
  return chosenCandidates;
}

function showVotingConfirmationModal(
  isRankedChoice,
  votingId,
  chosenCandidates,
  votingPassword
) {

  function getVotingConfirmationModalInstance() {
    const modalEle = document.getElementById("voting-modal");
    return M.Modal.getInstance(modalEle);
  }

  function setVotingConfirmationEventListener(e) {
    const form = getVotingForm(votingId);
    Array.from(
      form.querySelectorAll("input[name=candidate]:checked")
    ).forEach((elem) => elem.setAttribute("disabled", true));
    const modalInstance = getVotingConfirmationModalInstance();
    const confirmationModalTextArea = document.getElementById("voting-modal-text");
    const closeModalButton = document.getElementById("voting-modal-close");

    const data = {
      candidates: isRankedChoice
        ? chosenCandidates.map((c) => `${c.id}:${c.position}`)
        : chosenCandidates.map((c) => c.id),
      voting_password: votingPassword,
    };

    closeModalButton.setAttribute("disabled", true);
    e.target.setAttribute("disabled", true);

    callApi(
      `${SITE_ROOT_PATH}votings/${votingId}/${
        isRankedChoice ? "vote-ranked-choice" : "vote-normal"
      }/`,
      "POST",
      data
    )
      .then((res) => {
        if (!res.ok) {
          if (res.status == 403) {
            throw Error(
              "Äänestäminen epäonnistui. Tarkista äänestyksen salasana."
            );
          }
          throw Error(
            "Äänestäminen epäonnistui. Päivitä sivu ja yritä uudelleen!"
          );
        }
        return res.text();
      })
      .then((html) => {
        showUserNotification(
          USER_NOTIFICATION.CONFIRMATION,
          "Äänestäminen onnistui. Päivitetään äänestysluettelo."
        );
        // Do not distract the user with things happening too fast
        window.setTimeout(() => {
          modalInstance.close();
          updateVotingListFromHtml(html);
          e.target.removeAttribute("disabled");
          closeModalButton.removeAttribute("disabled");
        } , 500);
      })
      .catch((error) => {
        confirmationModalTextArea.innerHTML =
          error.message.length > 0
            ? error.message
            : "Äänestäminen saattoi epäonnistua. Päivitä sivu ja tarkista,\
            näkyykö äänestys vielä äänestämättömänä.";
        e.target.removeAttribute("disabled");
        closeModalButton.removeAttribute("disabled");
      });

    confirmationModalTextArea.innerHTML = "Äänesi on lähetetty. Odotetaan vahvistusta äänestyspalvelimelta. \
    Jos mitään ei tapahdu 30 sekunnin kuluessa, päivitä sivu.";
  }

  // Initialize modal
  const modal = document.getElementById("voting-modal");
  const btnConfirmation = modal.querySelector("#voting-modal-confirm");

  // Remove event listener on modal close to prevent
  // multiple votes. This is also handled in the backend
  // via a custom SessionLockMiddleware
  M.Modal.init(modal, {
    onCloseEnd: () =>
      btnConfirmation.removeEventListener(
        "click",
        setVotingConfirmationEventListener
      ),
    dismissible: false,
  });
  const instance = M.Modal.getInstance(modal);

  // Set modal content
  const candidatesString = chosenCandidates
    .map((c) => (isRankedChoice ? `${c.position}.${c.name}` : c.name))
    .join(", ");
  const singularOrPlural =
    chosenCandidates.length > 1 ? "ehdokkaita:\n" : "ehdokasta:\n";
  document.getElementById(
    "voting-modal-text"
  ).innerHTML = `Olet äänestämässä ${singularOrPlural} ${candidatesString}`;

  btnConfirmation.addEventListener("click", setVotingConfirmationEventListener);
  instance.open();
}

function getVotingPasswordTyped(votingId) {
  const passwordField = document.getElementById(`voting-password-${votingId}`);
  if (passwordField) {
    return passwordField.value;
  }
  return "";
  // Empty password corresponds to "no input" which can always be sent.
}

function vote(votingId) {
  const chosenCandidates = getChosenCandidates(false, votingId);
  const votingPassword = getVotingPasswordTyped(votingId);
  if (chosenCandidates.length === 0) {
    showUserNotification(
      USER_NOTIFICATION.WARNING,
      "Valitse ainakin yksi ehdokas."
    );
    return;
  }
  showVotingConfirmationModal(
    false,
    votingId,
    chosenCandidates,
    votingPassword
  );
}

function RankedChoiceVoteElection(votingId) {
  const chosenCandidates = getChosenCandidates(true, votingId).sort(
    compareChosenCandidates
  );
  const votingPassword = getVotingPasswordTyped(votingId);
  showVotingConfirmationModal(true, votingId, chosenCandidates, votingPassword);
}

// Currently only used in admin mode
async function refreshVotingList(admin = false) {
  const votingListRefreshButton = document.getElementById("voting-list-refresh-button");
  const votingListRefreshButtonText = votingListRefreshButton.innerHTML;
  const votingArea = document.getElementById("voting-list-area");
  const adminPath = admin ? "admin/" : "";
  const failMsg = "Äänestysten haku ei onnistunut. Päivitä sivu. Jos koetit äänestää, katso, näkyykö äänestys jo äänestettynä.";

  votingListRefreshButton.innerHTML = "Päivitetään...";
  votingListRefreshButton.disabled = true;

  if (!admin) {
    votingArea.innerHTML = "<p>Äänestysluetteloa päivitetään parhaillaan.</p>";
  }

  await callApi(`${SITE_ROOT_PATH}${adminPath}votings/list/`, "GET")
    .then((res) => res.text())
    .then((html) => (votingArea.innerHTML = html))
    .catch(() => {
      showUserNotification(
        USER_NOTIFICATION.WARNING,
        failMsg
      );
      votingArea.innerHTML = "<p>" + failMsg + "</p>";
    });

  votingListRefreshButton.innerHTML = "Luettelo päivitetty!"

  // Prevent from clicking the button unnecessarily many times within a short time
  var timeoutDuration = 5000;
  if (admin) {
    timeoutDuration = 1000;
  }
  window.setTimeout(() => {
    votingListRefreshButton.innerHTML = votingListRefreshButtonText;
    votingListRefreshButton.disabled = false;
    votingListRefreshButton.blur();
  }, timeoutDuration);
  setupEventListeners();
  resetCandidateOrder();
}

function updateVotingListFromHtml(html) {
  const votingArea = document.getElementById("voting-list-area");
  votingArea.innerHTML = html;
  setupEventListeners();
  resetCandidateOrder();
}

function selectVote(elem, votingId) {
  const form = document.getElementById(`voting-form-${votingId}`);
  const maxVotes = parseInt(form.getAttribute("data-voting-max-votes"));
  elem.checked = true;
  elem.nextElementSibling.classList.remove("blue-grey");
  elem.nextElementSibling.classList.add("prodeko-blue");

  const formInputs = form.querySelectorAll(
    `input[type=${maxVotes > 1 ? "checkbox" : "radio"}]`
  );
  const givenVotes = Array.from(formInputs).filter((c) => c.checked).length;
  formInputs.forEach((c) => {
    if (givenVotes === maxVotes) {
      if (c.id !== elem.id) {
        c.checked = false;
        c.nextElementSibling.classList.remove("prodeko-blue");
        c.nextElementSibling.classList.add("blue-grey");
      }
    }
  });
}

// User login / logout
function logout() {
  callApi(`${SITE_ROOT_PATH}user/logout/`, "POST")
    .then((res) => res.json())
    .then((data) => {
      if (data.status === 0) {
        document.cookie = "";
        location.reload();
        localStorage.removeItem("token");
      }
    })
    .catch(() =>
      showUserNotification(
        USER_NOTIFICATION.WARNING,
        "Uloskirjautuminen epäonnistui. Päivitä sivu."
      )
    );
}

function submitToken() {
  var token = document.getElementById("type-token-field").value;
  var alias = document.getElementById("type-alias-field").value;
  const notificationArea = document.getElementById("login-notification-area");

  notificationArea.classList.add("loading-token-notification");
  notificationArea.classList.remove("wrong-token-warning");
  notificationArea.innerHTML = "Ladataan... &#129312";
  document.cookie = csrftoken = jQuery("[name=csrfmiddlewaretoken]").val();

  callApi(`${SITE_ROOT_PATH}user/login/`, "POST", {
    token: token,
    alias: alias,
  })
    .then((res) => {
      if (!res.ok) {
        if (res.status === 401) {
          throw Error("Virheellinen kirjautumiskoodi");
        } else if (res.status === 403) {
          throw Error("Alias ei ole sallittu, tai se on jo varattu");
        }
        throw Error("Kirjautuminen epäonnistui");
      }
      location.reload();
    })
    .catch((error) =>
      window.setTimeout(() => {
        notificationArea.classList.remove("loading-token-notification");
        notificationArea.classList.add("wrong-token-warning");
        notificationArea.innerHTML = error.message;
      }, 100)
    );
  localStorage.setItem("token", token);
}

// Admin
function generateTokens(count) {
  const generateTokensButton = document.getElementById(
    "generate-tokens-button"
  );
  generateTokensButton.setAttribute("disabled", true);

  callApi(`${SITE_ROOT_PATH}admin/tokens/generate/`, "POST", { count })
    .then(() => {
      showUserNotification(
        USER_NOTIFICATION.CONFIRMATION,
        "Koodien generointi onnistui."
      );
      setTimeout(() => location.reload(), 500);
    })
    .catch(() =>
      showUserNotification(
        USER_NOTIFICATION.WARNING,
        "Koodien generointi epäonnistui."
      )
    );
  generateTokensButton.removeAttribute("disabled");
}

function activateOrInvalidateToken(isActivate, code, number) {
  var token = code;

  if (token.length < 1) {
    return;
  }

  var button = document.getElementById(
    `${isActivate ? "activate" : "invalidate"}-token-button-${number}`
  );
  var clickedState = button.dataset["clicked"];

  // Require two clicks to activate code
  if (clickedState == "0") {
    button.innerHTML = isActivate ? "Aktivoi?" : "Mitätöi?";
    button.classList.add("orange");
    button.classList.remove(isActivate ? "green" : "red");
    button.dataset["clicked"] = "1";
    return;
  }

  button.setAttribute("disabled", true);

  callApi(
    `${SITE_ROOT_PATH}admin/tokens/${isActivate ? "activate" : "invalidate"}/`,
    "POST",
    { token }
  )
    .then(() => location.reload())
    .catch(() =>
      showUserNotification(
        USER_NOTIFICATION.WARNING,
        `Koodin ${
          isActivate ? "aktivointi" : "mitätöinti"
        } epäonnistui. Tarkista koodi.`
      )
    );
}

function createVoting() {
  const isRankedChoice = document.getElementById("is-ranked-choice-election")
    .checked;
  const isPasswordProtected = document.getElementById(
    "voting-add-is-password-protected"
  ).checked;
  const votingName = document.getElementById("voting-name").value;
  const votingDescription = document.getElementById("voting-description").value;
  const votingPassword = document.getElementById("voting-add-voting-password")
    .value;
  const maxVotes = document.getElementById("max-votes").value;

  const data = {
    is_ranked_choice: isRankedChoice,
    is_password_protected: isPasswordProtected,
    voting_name: votingName,
    voting_description: votingDescription,
    voting_password: votingPassword,
    max_votes: maxVotes,
  };
  callApi(`${SITE_ROOT_PATH}admin/votings/create/`, "POST", data)
    .then(() => refreshVotingList(true))
    .catch(() =>
      showUserNotification(
        USER_NOTIFICATION.WARNING,
        "Äänestyksen luominen ei ehkä onnistunut! Päivitä sivu!"
      )
    );
}

function addCandidate(votingId, isRankedChoice) {
  const candidate = document.getElementById(`voting-${votingId}-candidate-name`)
    .value;
  if (candidate) {
    const data = {
      is_ranked_choice: isRankedChoice,
      candidate_name: candidate,
    };
    callApi(`${SITE_ROOT_PATH}admin/votings/${votingId}/add/`, "POST", data)
      .then(() => refreshVotingList(true))
      .catch(() =>
        showUserNotification(
          USER_NOTIFICATION.WARNING,
          "Ehdokkaan lisääminen ei ehkä onnistunut! Päivitä sivu!"
        )
      );
  }
}

function removeCandidate(candidate_id, is_ranked_choice) {
  const data = {
    is_ranked_choice,
  };
  callApi(
    `${SITE_ROOT_PATH}admin/votings/${candidate_id}/remove/`,
    "POST",
    data
  )
    .then(() => refreshVotingList(true))
    .catch(() =>
      showUserNotification(
        USER_NOTIFICATION.WARNING,
        "Äänestyksen luominen ei ehkä onnistunut! Päivitä sivu!"
      )
    );
}

function closeVoting(votingId, is_ranked_choice) {
  const data = {
    is_ranked_choice,
  };
  callApi(`${SITE_ROOT_PATH}admin/votings/${votingId}/close/`, "POST", data)
    .then(async (res) => {
      const data = await res.json();
      if (res.status !== 200) {
        if (data.message) {
          showUserNotification(USER_NOTIFICATION.WARNING, data.message);
        }
      }
      return data;
    })
    .then((data) => {
      refreshVotingList(true);
      // If a sound is already playing, reveal the result with a badum-tss sound
      if (SOUND_STATE !== 0) {
        playSound(3);
      }
    })
    .catch((err) =>
      showUserNotification(
        USER_NOTIFICATION.WARNING,
        "Äänestyksen sulkeminen ei ehkä onnistunut! Päivitä sivu!"
      )
    );
}

function openVoting(votingId, is_ranked_choice) {
  const data = {
    is_ranked_choice,
  };
  callApi(`${SITE_ROOT_PATH}admin/votings/${votingId}/open/`, "POST", data)
    .then(() => refreshVotingList(true))
    .catch(() =>
      showUserNotification(
        USER_NOTIFICATION.WARNING,
        "Äänestyksen avaaminen ei ehkä onnistunut! Päivitä sivu!"
      )
    );
}

// Hakutaulukon funktioi

function searchFunction() {
  // Declare variables
  var input, filter, table, tr, td, i;
  input = document.getElementById("search");
  filter = input.value.toUpperCase();
  table = document.getElementById("search-table");
  tr = table.getElementsByTagName("tr");

  // Loop through all table rows, and hide those who don't match the search query
  for (i = 0; i < tr.length; i++) {
    td = tr[i].getElementsByTagName("td")[0];
    if (td) {
      if (td.innerHTML.toUpperCase().indexOf(filter) > -1) {
        tr[i].style.display = "";
      } else {
        tr[i].style.display = "none";
      }
    }
  }
}

function invalidateActiveTokens() {
  callApi(`${SITE_ROOT_PATH}admin/tokens/invalidate/all/`, "POST")
    .then(() => location.reload())
    .catch(() =>
      showUserNotification(
        USER_NOTIFICATION.WARNING,
        "Koodien mitätöinti epäonnistui!"
      )
    );
}

function stopAllSounds() {
  document.querySelectorAll(".sound-track").forEach((track) => {
    track.pause();
    track.currentTime = 0;
  });
}

function playSound(trackNo) {
  var track = document.getElementById(`sound-track-${trackNo}`);

  stopAllSounds();

  if (SOUND_STATE !== trackNo) {
    track.play();
    SOUND_STATE = trackNo;
  } else {
    SOUND_STATE = 0;
  }
}

// Reset chosen ordering of candidates in UI when using STV vote, for particular votingId
function clearVotes(votingId) {
  const form = getVotingForm(votingId);
  Array.from(form.querySelectorAll(".voting-order")).forEach((elem) => {
    elem.innerHTML = "-";
    elem.nextElementSibling.classList.remove("prodeko-blue");
    elem.nextElementSibling.classList.add("blue-grey");
  });
  resetCandidateOrder();
}

function resetCandidateOrder() {
  votesGiven = 0;
  currentVotingId = -1;
}

function setupEventListeners() {
  var transferVoteCandidates = document.getElementsByClassName(
    "transfer-vote-candidate"
  );

  Array.from(transferVoteCandidates).forEach((candidate) => {
    candidate.addEventListener("click", (e) => {
      const candidate = e.target.getAttribute("value");
      const votingId = e.target.getAttribute("value").split("-")[2];
      const form = getVotingForm(votingId);
      const candidateCount = form.querySelectorAll("label").length;
      currentVotingId = votingId;

      const getOrderLabel = () => document.getElementById(candidate).innerHTML;
      var label = getOrderLabel();

      if (label === "-") {
        if (currentVotingId !== votingId) {
          currentVotingId = votingId;
          votesGiven = 0;
        } else if (votesGiven < candidateCount) {
          votesGiven += 1;
          const candidateRank = document.getElementById(candidate);
          candidateRank.innerHTML = votesGiven;
          candidateRank.nextElementSibling.classList.remove("blue-grey");
          candidateRank.nextElementSibling.classList.add("prodeko-blue");
        }
      } else {
        const clickedCandidate = e.target;
        const clickedRank = clickedCandidate.previousElementSibling.innerHTML;

        clickedCandidate.classList.remove("prodeko-blue");
        clickedCandidate.classList.add("blue-grey");
        clickedCandidate.previousElementSibling.innerHTML = "-";
        Array.from(form.querySelectorAll(".voting-order")).forEach((elem) => {
          const rank = parseInt(elem.innerHTML);
          if (rank > clickedRank) {
            elem.innerHTML = rank - 1;
          }
        });
        votesGiven -= 1;
      }
    });
  });
}

function validateAliasSyntax(aliasInput) {
  const aliasRegex = /^[A-Z0-9\u00C0-\u00D6\u00D8-\u00DE][A-Z0-9\u00C0-\u00D6\u00D8-\u00DE_-]+$/;
  const lengthOk = aliasInput.length >= 3 && aliasInput.length <= 20;
  return lengthOk && aliasRegex.test(aliasInput.toUpperCase());
}

function instaValidateAliasSyntax(formField) {
  const fieldClasses = formField.classList;
  if (validateAliasSyntax(formField.value)) {
    fieldClasses.add("valid");
    fieldClasses.remove("invalid");
  } else {
    fieldClasses.add("invalid");
    fieldClasses.remove("valid");
  }
}

function expandResults(element) {
  element.parentNode.classList.toggle("expanded");
}

function toggleNotVotedList(tableId) {
  document.getElementById(tableId).classList.toggle("hide");
}
