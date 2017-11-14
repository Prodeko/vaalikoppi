
// Kaikenlaisia funktioita äänestykseen
// Tarvii jQueryn


SITE_ROOT_PATH = '/vaalikoppi/';

function vote(votingId) {

	var form = $('#voting-form-' + votingId);
	var selectedCandidateId = form.find('input[name=candidate]:checked').val();
	var candidateId = form.find('input[name=candidate]:checked').attr('id')
	var candidateName = form.find('label[for=' + candidateId + ']').text()
	console.log(candidateName)

	if (selectedCandidateId == null) {
		return;
	}

	var confirmation = confirm('Olet äänestämässä ehdokasta: ' + candidateName);

	if (!confirmation) {
		return;
	}

	form.find('input, button').prop('disabled', true);

	var query = $.post(SITE_ROOT_PATH + 'votings/' + votingId + '/vote/',
		{ candidate : selectedCandidateId }
	).done(function(data) {
	}).fail(function(data) {
		alert('Äänestäminen epäonnistui. Päivitä sivu ja yritä uudelleen!');
	});

	refreshVotingList();

}

function refreshVotingList(admin = false) {

	var votingArea = $('#voting-list-area');

	var query = $.get(SITE_ROOT_PATH + (admin ? 'admin/' : '' ) + 'votings/list/', function(data) {
		votingArea.html(data);
	})
	.fail(function() {
		alert('Äänestysten haku ei onnistunut. Päivitä sivu. Jos koetit äänestää, katso, näkyykö äänestys jo äänestettynä.');
	});
}

function generateTokens(count) {

	$('#generate-tokens-button').prop('disabled', true);

	var query = $.post(SITE_ROOT_PATH + 'admin/tokens/generate/',
		{ count : count }
	).done(function(data) {
		alert('Koodien generointi onnistui.');
	}).fail(function(data) {
		alert('Koodien generointi epäonnistui.');
	});

	$('#generate-tokens-button').prop('disabled', false);
}

function invalidateToken() {

	var token = $('#invalidate-token-field').val();

	if (token.length < 1) {
		return;
	}

	$('#invalidate-code-button').prop('disabled', true);

	var query = $.post(SITE_ROOT_PATH + 'invalidate/',
		{ token : token }
	).done(function(data) {
		alert('Koodin invalidointi onnistui.');
	}).fail(function(data) {
		alert('Koodin invalidointi epäonnistui. Tarkista koodi.');
	});

	$('#invalidate-token-button').prop('disabled', false);
}

// Just for the UI. Everything is validated in the back-end...
function checkVoterStatus(callback) {

	var query = $.getJSON(SITE_ROOT_PATH + 'user/status/')
	.done(function(data) {
		try {
			// No token/non-active token
			if (data.status === 0) {
				callback([ false, '' ]);
			} else if (data.status === 1) {
				if (data.activated === true && data.invalidated === false) {
					$('#token_div').html(data.token);
					callback([ true, data.token ]);
				} else {
					callback([ false, data.token ]);
				}
			} else throw new Exception();
		} catch (err) {
			callback([ false, '' ]);
		}
	})
	.fail(function() {
		alert('Tilan haku palvelimelta ei onnistunut. Koeta päivittää sivu.');
	});
}

function submitToken() {

	var token = $('#type-token-field').val();
	var warning = $('#main-login-prompt .wrong-token-warning');

	warning.addClass('invisible');

	var query = $.post(SITE_ROOT_PATH + 'user/login/',
		{ token : token }
	).done(function(data) {
		toggleLoginPrompt();
		refreshVotingList();
	}).fail(function(data) {
		window.setTimeout(function() {
			warning.removeClass('invisible');
		}, 100);
	});
}

function toggleLoginPrompt() {
	$('#main-login-prompt').toggle();
	$('#voting-list-updater, #voting-list-area').fadeToggle();
}

function closeVoting(votingId) {

	var query = $.post(SITE_ROOT_PATH + 'admin/votings/' + votingId + '/close/').done(function(data) {
		refreshVotingList(true);
	}).fail(function(data) {
		alert('Äänestyksen sulkeminen ei ehkä onnistunut! Päivitä sivu!');
	});
}

function openVoting(votingId) {

	var query = $.post(SITE_ROOT_PATH + 'admin/votings/' + votingId + '/open/').done(function(data) {
		refreshVotingList(true);
	}).fail(function(data) {
		alert('Äänestyksen avaaminen ei ehkä onnistunut! Päivitä sivu!');
	});
}
